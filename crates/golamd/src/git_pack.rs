#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::time::Duration;

use miniz_oxide::inflate::stream::{InflateState, inflate};
use miniz_oxide::{DataFormat, MZError, MZFlush, MZStatus};

use crate::git_read_budget::{
    DECOMPRESSION_INPUT_QUANTUM_BYTES, DECOMPRESSION_OUTPUT_QUANTUM_BYTES,
    DecompressionBudgetError, DecompressionDeadline,
};
use crate::git_sha1::{GitObjectSha1, GitObjectSha1Error};

const SHA1_BYTES: usize = 20;
const PACK_HEADER_BYTES: usize = 12;
const PACK_TRAILER_BYTES: usize = SHA1_BYTES;
const INDEX_HEADER_BYTES: usize = 8;
const INDEX_FANOUT_BYTES: usize = 256 * 4;
const INDEX_TRAILER_BYTES: usize = SHA1_BYTES * 2;
const PACK_INDEX_MAGIC: [u8; 4] = [0xff, b't', b'O', b'c'];

pub const MAX_PACK_INDEX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_PACK_BYTES: usize = 256 * 1024 * 1024;
pub const MAX_PACK_OBJECTS: usize = 250_000;
pub const MAX_PACK_OBJECT_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_DELTA_DEPTH: usize = 64;
pub const DEFAULT_PACK_READ_TIME_BUDGET: Duration = Duration::from_secs(10);
pub const MAX_PACK_READ_TIME_BUDGET: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitPackBounds {
    pub max_index_bytes: usize,
    pub max_pack_bytes: usize,
    pub max_objects: usize,
    pub max_object_bytes: usize,
    pub max_delta_depth: usize,
    pub max_duration: Duration,
}

impl Default for GitPackBounds {
    fn default() -> Self {
        Self {
            max_index_bytes: MAX_PACK_INDEX_BYTES,
            max_pack_bytes: MAX_PACK_BYTES,
            max_objects: MAX_PACK_OBJECTS,
            max_object_bytes: MAX_PACK_OBJECT_BYTES,
            max_delta_depth: MAX_DELTA_DEPTH,
            max_duration: DEFAULT_PACK_READ_TIME_BUDGET,
        }
    }
}

impl GitPackBounds {
    pub fn validate(self) -> Result<(), GitPackError> {
        let minimum_index = INDEX_HEADER_BYTES + INDEX_FANOUT_BYTES + INDEX_TRAILER_BYTES;
        if self.max_index_bytes < minimum_index
            || self.max_index_bytes > MAX_PACK_INDEX_BYTES
            || self.max_pack_bytes < PACK_HEADER_BYTES + PACK_TRAILER_BYTES
            || self.max_pack_bytes > MAX_PACK_BYTES
            || self.max_objects == 0
            || self.max_objects > MAX_PACK_OBJECTS
            || self.max_object_bytes == 0
            || self.max_object_bytes > MAX_PACK_OBJECT_BYTES
            || self.max_delta_depth == 0
            || self.max_delta_depth > MAX_DELTA_DEPTH
            || self.max_duration.is_zero()
            || self.max_duration > MAX_PACK_READ_TIME_BUDGET
        {
            return Err(GitPackError::InvalidBounds);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackObjectId([u8; SHA1_BYTES]);

impl PackObjectId {
    pub const fn from_bytes(bytes: [u8; SHA1_BYTES]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; SHA1_BYTES] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackedObjectKind {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl PackedObjectKind {
    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Tree => "tree",
            Self::Blob => "blob",
            Self::Tag => "tag",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedGitObject {
    pub id: PackObjectId,
    pub kind: PackedObjectKind,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitPackIndexEntry {
    pub object_id: PackObjectId,
    pub crc32: u32,
    pub offset: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitPackIndex {
    pub entries: Vec<GitPackIndexEntry>,
    pub pack_checksum: [u8; SHA1_BYTES],
    pub index_checksum: [u8; SHA1_BYTES],
}

pub fn parse_pack_index_v2(
    bytes: &[u8],
    bounds: GitPackBounds,
) -> Result<GitPackIndex, GitPackError> {
    bounds.validate()?;
    if bytes.len() > bounds.max_index_bytes {
        return Err(GitPackError::IndexByteLimitExceeded);
    }
    if bytes.len() < INDEX_HEADER_BYTES + INDEX_FANOUT_BYTES + INDEX_TRAILER_BYTES {
        return Err(GitPackError::TruncatedIndex);
    }

    let content_len = bytes
        .len()
        .checked_sub(SHA1_BYTES)
        .ok_or(GitPackError::TruncatedIndex)?;
    let index_checksum: [u8; SHA1_BYTES] = bytes[content_len..]
        .try_into()
        .map_err(|_| GitPackError::TruncatedIndex)?;
    if GitObjectSha1::digest(&bytes[..content_len])? != index_checksum {
        return Err(GitPackError::IndexChecksumMismatch);
    }

    let mut cursor = Cursor::new(&bytes[..content_len]);
    if cursor.take(4)? != PACK_INDEX_MAGIC {
        return Err(GitPackError::InvalidIndexMagic);
    }
    let version = cursor.read_u32()?;
    if version != 2 {
        return Err(GitPackError::UnsupportedIndexVersion(version));
    }

    let mut fanout = [0_u32; 256];
    let mut previous = 0_u32;
    for slot in &mut fanout {
        let value = cursor.read_u32()?;
        if value < previous {
            return Err(GitPackError::InvalidFanout);
        }
        *slot = value;
        previous = value;
    }
    let object_count =
        usize::try_from(fanout[255]).map_err(|_| GitPackError::ObjectLimitExceeded)?;
    if object_count > bounds.max_objects {
        return Err(GitPackError::ObjectLimitExceeded);
    }

    let mut object_ids = Vec::with_capacity(object_count);
    for _ in 0..object_count {
        let id = PackObjectId::from_bytes(
            cursor
                .take(SHA1_BYTES)?
                .try_into()
                .map_err(|_| GitPackError::TruncatedIndex)?,
        );
        if object_ids.last().is_some_and(|previous| *previous >= id) {
            return Err(GitPackError::ObjectIdsNotStrictlySorted);
        }
        object_ids.push(id);
    }
    validate_fanout(&fanout, &object_ids)?;

    let mut crcs = Vec::with_capacity(object_count);
    for _ in 0..object_count {
        crcs.push(cursor.read_u32()?);
    }

    let mut raw_offsets = Vec::with_capacity(object_count);
    let mut large_offset_count = 0_usize;
    for _ in 0..object_count {
        let raw = cursor.read_u32()?;
        if raw & 0x8000_0000 != 0 {
            large_offset_count = large_offset_count
                .checked_add(1)
                .ok_or(GitPackError::OffsetTableInvalid)?;
        }
        raw_offsets.push(raw);
    }

    let mut large_offsets = Vec::with_capacity(large_offset_count);
    for _ in 0..large_offset_count {
        large_offsets.push(cursor.read_u64()?);
    }

    let pack_checksum: [u8; SHA1_BYTES] = cursor
        .take(SHA1_BYTES)?
        .try_into()
        .map_err(|_| GitPackError::TruncatedIndex)?;
    if cursor.remaining() != 0 {
        return Err(GitPackError::IndexTrailingData);
    }

    let mut referenced_large = HashSet::with_capacity(large_offset_count);
    let mut entries = Vec::with_capacity(object_count);
    for ((object_id, crc32), raw_offset) in object_ids
        .into_iter()
        .zip(crcs.into_iter())
        .zip(raw_offsets.into_iter())
    {
        let offset = if raw_offset & 0x8000_0000 == 0 {
            u64::from(raw_offset)
        } else {
            let table_index = usize::try_from(raw_offset & 0x7fff_ffff)
                .map_err(|_| GitPackError::OffsetTableInvalid)?;
            let value = *large_offsets
                .get(table_index)
                .ok_or(GitPackError::OffsetTableInvalid)?;
            if value <= u64::from(0x7fff_ffff_u32) || !referenced_large.insert(table_index) {
                return Err(GitPackError::OffsetTableInvalid);
            }
            value
        };
        entries.push(GitPackIndexEntry {
            object_id,
            crc32,
            offset,
        });
    }
    if referenced_large.len() != large_offset_count
        || (0..large_offset_count).any(|index| !referenced_large.contains(&index))
    {
        return Err(GitPackError::OffsetTableInvalid);
    }

    Ok(GitPackIndex {
        entries,
        pack_checksum,
        index_checksum,
    })
}

pub fn read_packed_object(
    pack: &[u8],
    index: &GitPackIndex,
    wanted: PackObjectId,
    bounds: GitPackBounds,
) -> Result<PackedGitObject, GitPackError> {
    bounds.validate()?;
    validate_pack(pack, index, bounds)?;

    let wanted_index = index
        .entries
        .binary_search_by_key(&wanted, |entry| entry.object_id)
        .map_err(|_| GitPackError::MissingPackedObject(wanted))?;
    let lookup = PackLookup::new(pack, index)?;
    let mut active_offsets = HashSet::new();
    resolve_entry(
        pack,
        index,
        &lookup,
        wanted_index,
        bounds,
        0,
        &mut active_offsets,
    )
}

fn validate_pack(
    pack: &[u8],
    index: &GitPackIndex,
    bounds: GitPackBounds,
) -> Result<(), GitPackError> {
    if pack.len() > bounds.max_pack_bytes {
        return Err(GitPackError::PackByteLimitExceeded);
    }
    if pack.len() < PACK_HEADER_BYTES + PACK_TRAILER_BYTES {
        return Err(GitPackError::TruncatedPack);
    }
    if &pack[..4] != b"PACK" {
        return Err(GitPackError::InvalidPackMagic);
    }
    let version = u32::from_be_bytes(
        pack[4..8]
            .try_into()
            .map_err(|_| GitPackError::TruncatedPack)?,
    );
    if version != 2 {
        return Err(GitPackError::UnsupportedPackVersion(version));
    }
    let declared_count = usize::try_from(u32::from_be_bytes(
        pack[8..12]
            .try_into()
            .map_err(|_| GitPackError::TruncatedPack)?,
    ))
    .map_err(|_| GitPackError::ObjectLimitExceeded)?;
    if declared_count > bounds.max_objects || declared_count != index.entries.len() {
        return Err(GitPackError::PackObjectCountMismatch);
    }

    let trailer_start = pack.len() - PACK_TRAILER_BYTES;
    let pack_checksum: [u8; SHA1_BYTES] = pack[trailer_start..]
        .try_into()
        .map_err(|_| GitPackError::TruncatedPack)?;
    if GitObjectSha1::digest(&pack[..trailer_start])? != pack_checksum {
        return Err(GitPackError::PackChecksumMismatch);
    }
    if pack_checksum != index.pack_checksum {
        return Err(GitPackError::PackIndexChecksumMismatch);
    }
    Ok(())
}

struct PackLookup {
    offset_to_entry: HashMap<u64, usize>,
    sorted_offsets: Vec<u64>,
    trailer_start: u64,
}

impl PackLookup {
    fn new(pack: &[u8], index: &GitPackIndex) -> Result<Self, GitPackError> {
        let trailer_start = u64::try_from(pack.len() - PACK_TRAILER_BYTES)
            .map_err(|_| GitPackError::PackOffsetInvalid)?;
        let mut offset_to_entry = HashMap::with_capacity(index.entries.len());
        let mut sorted_offsets = Vec::with_capacity(index.entries.len());
        for (entry_index, entry) in index.entries.iter().enumerate() {
            if entry.offset < PACK_HEADER_BYTES as u64
                || entry.offset >= trailer_start
                || offset_to_entry.insert(entry.offset, entry_index).is_some()
            {
                return Err(GitPackError::PackOffsetInvalid);
            }
            sorted_offsets.push(entry.offset);
        }
        sorted_offsets.sort_unstable();
        Ok(Self {
            offset_to_entry,
            sorted_offsets,
            trailer_start,
        })
    }

    fn end_for(&self, offset: u64) -> Result<u64, GitPackError> {
        let position = self
            .sorted_offsets
            .binary_search(&offset)
            .map_err(|_| GitPackError::PackOffsetInvalid)?;
        Ok(self
            .sorted_offsets
            .get(position + 1)
            .copied()
            .unwrap_or(self.trailer_start))
    }
}

fn resolve_entry(
    pack: &[u8],
    index: &GitPackIndex,
    lookup: &PackLookup,
    entry_index: usize,
    bounds: GitPackBounds,
    depth: usize,
    active_offsets: &mut HashSet<u64>,
) -> Result<PackedGitObject, GitPackError> {
    if depth >= bounds.max_delta_depth {
        return Err(GitPackError::DeltaDepthExceeded);
    }
    let index_entry = *index
        .entries
        .get(entry_index)
        .ok_or(GitPackError::PackOffsetInvalid)?;
    if !active_offsets.insert(index_entry.offset) {
        return Err(GitPackError::DeltaCycle);
    }

    let result = (|| {
        let entry_end = lookup.end_for(index_entry.offset)?;
        let start =
            usize::try_from(index_entry.offset).map_err(|_| GitPackError::PackOffsetInvalid)?;
        let end = usize::try_from(entry_end).map_err(|_| GitPackError::PackOffsetInvalid)?;
        let entry_bytes = pack
            .get(start..end)
            .ok_or(GitPackError::PackOffsetInvalid)?;
        if crc32(entry_bytes) != index_entry.crc32 {
            return Err(GitPackError::PackedEntryCrcMismatch);
        }

        let header =
            parse_pack_entry_header(entry_bytes, index_entry.offset, bounds.max_object_bytes)?;
        let (kind, body) = match header.representation {
            PackRepresentation::Base(kind) => {
                let (body, consumed) = inflate_one_zlib(
                    &entry_bytes[header.payload_offset..],
                    bounds.max_object_bytes,
                    bounds.max_duration,
                )?;
                if consumed != entry_bytes.len() - header.payload_offset
                    || body.len() != header.representation_size
                {
                    return Err(GitPackError::PackedRepresentationSizeMismatch);
                }
                (kind, body)
            }
            PackRepresentation::OfsDelta(base_offset) => {
                let base_index = *lookup
                    .offset_to_entry
                    .get(&base_offset)
                    .ok_or(GitPackError::MissingDeltaBaseOffset(base_offset))?;
                let base = resolve_entry(
                    pack,
                    index,
                    lookup,
                    base_index,
                    bounds,
                    depth + 1,
                    active_offsets,
                )?;
                let (delta, consumed) = inflate_one_zlib(
                    &entry_bytes[header.payload_offset..],
                    bounds.max_object_bytes,
                    bounds.max_duration,
                )?;
                if consumed != entry_bytes.len() - header.payload_offset
                    || delta.len() != header.representation_size
                {
                    return Err(GitPackError::PackedRepresentationSizeMismatch);
                }
                let body = apply_delta(&base.bytes, &delta, bounds.max_object_bytes)?;
                (base.kind, body)
            }
            PackRepresentation::RefDelta(base_id) => {
                let base_index = index
                    .entries
                    .binary_search_by_key(&base_id, |entry| entry.object_id)
                    .map_err(|_| GitPackError::ThinPackUnsupported(base_id))?;
                let base = resolve_entry(
                    pack,
                    index,
                    lookup,
                    base_index,
                    bounds,
                    depth + 1,
                    active_offsets,
                )?;
                let (delta, consumed) = inflate_one_zlib(
                    &entry_bytes[header.payload_offset..],
                    bounds.max_object_bytes,
                    bounds.max_duration,
                )?;
                if consumed != entry_bytes.len() - header.payload_offset
                    || delta.len() != header.representation_size
                {
                    return Err(GitPackError::PackedRepresentationSizeMismatch);
                }
                let body = apply_delta(&base.bytes, &delta, bounds.max_object_bytes)?;
                (base.kind, body)
            }
        };

        if body.len() > bounds.max_object_bytes {
            return Err(GitPackError::ObjectSizeLimitExceeded);
        }
        let actual_id = canonical_object_id(kind, &body)?;
        if actual_id != index_entry.object_id {
            return Err(GitPackError::PackedObjectHashMismatch);
        }
        Ok(PackedGitObject {
            id: index_entry.object_id,
            kind,
            bytes: body,
        })
    })();

    active_offsets.remove(&index_entry.offset);
    result
}

struct ParsedPackHeader {
    representation: PackRepresentation,
    representation_size: usize,
    payload_offset: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PackRepresentation {
    Base(PackedObjectKind),
    OfsDelta(u64),
    RefDelta(PackObjectId),
}

fn parse_pack_entry_header(
    entry: &[u8],
    absolute_offset: u64,
    max_object_bytes: usize,
) -> Result<ParsedPackHeader, GitPackError> {
    let first = *entry.first().ok_or(GitPackError::TruncatedPackEntry)?;
    let type_id = (first >> 4) & 0x07;
    let mut size = u64::from(first & 0x0f);
    let mut shift = 4_u32;
    let mut cursor = 1_usize;
    let mut current = first;
    let mut continuation_bytes = 0_usize;
    while current & 0x80 != 0 {
        continuation_bytes += 1;
        if continuation_bytes > 9 || shift >= 64 {
            return Err(GitPackError::InvalidPackObjectHeader);
        }
        current = *entry.get(cursor).ok_or(GitPackError::TruncatedPackEntry)?;
        cursor += 1;
        size |= u64::from(current & 0x7f)
            .checked_shl(shift)
            .ok_or(GitPackError::InvalidPackObjectHeader)?;
        shift += 7;
    }
    let representation_size =
        usize::try_from(size).map_err(|_| GitPackError::ObjectSizeLimitExceeded)?;
    if representation_size > max_object_bytes {
        return Err(GitPackError::ObjectSizeLimitExceeded);
    }

    let representation = match type_id {
        1 => PackRepresentation::Base(PackedObjectKind::Commit),
        2 => PackRepresentation::Base(PackedObjectKind::Tree),
        3 => PackRepresentation::Base(PackedObjectKind::Blob),
        4 => PackRepresentation::Base(PackedObjectKind::Tag),
        6 => {
            let (distance, consumed) = parse_ofs_delta_distance(&entry[cursor..])?;
            cursor = cursor
                .checked_add(consumed)
                .ok_or(GitPackError::PackOffsetInvalid)?;
            let base_offset = absolute_offset
                .checked_sub(distance)
                .ok_or(GitPackError::InvalidOfsDelta)?;
            if base_offset >= absolute_offset {
                return Err(GitPackError::InvalidOfsDelta);
            }
            PackRepresentation::OfsDelta(base_offset)
        }
        7 => {
            let base: [u8; SHA1_BYTES] = entry
                .get(cursor..cursor + SHA1_BYTES)
                .ok_or(GitPackError::TruncatedPackEntry)?
                .try_into()
                .map_err(|_| GitPackError::TruncatedPackEntry)?;
            cursor += SHA1_BYTES;
            PackRepresentation::RefDelta(PackObjectId::from_bytes(base))
        }
        _ => return Err(GitPackError::UnsupportedPackObjectType(type_id)),
    };
    if cursor >= entry.len() {
        return Err(GitPackError::TruncatedPackEntry);
    }
    Ok(ParsedPackHeader {
        representation,
        representation_size,
        payload_offset: cursor,
    })
}

fn parse_ofs_delta_distance(bytes: &[u8]) -> Result<(u64, usize), GitPackError> {
    let first = *bytes.first().ok_or(GitPackError::TruncatedPackEntry)?;
    let mut value = u64::from(first & 0x7f);
    let mut current = first;
    let mut consumed = 1_usize;
    while current & 0x80 != 0 {
        if consumed >= 10 {
            return Err(GitPackError::InvalidOfsDelta);
        }
        current = *bytes
            .get(consumed)
            .ok_or(GitPackError::TruncatedPackEntry)?;
        consumed += 1;
        value = value
            .checked_add(1)
            .and_then(|value| value.checked_shl(7))
            .and_then(|value| value.checked_add(u64::from(current & 0x7f)))
            .ok_or(GitPackError::InvalidOfsDelta)?;
    }
    if value == 0 {
        return Err(GitPackError::InvalidOfsDelta);
    }
    Ok((value, consumed))
}

fn inflate_one_zlib(
    compressed: &[u8],
    max_output_bytes: usize,
    max_duration: Duration,
) -> Result<(Vec<u8>, usize), GitPackError> {
    let deadline = DecompressionDeadline::start(max_duration)?;
    let mut state = InflateState::new(DataFormat::Zlib);
    let mut input_offset = 0_usize;
    let mut output = Vec::with_capacity(max_output_bytes.min(DECOMPRESSION_OUTPUT_QUANTUM_BYTES));

    loop {
        let input_end = input_offset
            .saturating_add(DECOMPRESSION_INPUT_QUANTUM_BYTES)
            .min(compressed.len());
        let input = &compressed[input_offset..input_end];
        let remaining = max_output_bytes.saturating_sub(output.len());
        let output_len = remaining.min(DECOMPRESSION_OUTPUT_QUANTUM_BYTES).max(1);
        let mut chunk = vec![0_u8; output_len];
        let result = deadline.run_quantum(input, &mut chunk, |input, output| {
            inflate(&mut state, input, output, MZFlush::None)
        })?;
        if result.bytes_consumed > input.len()
            || result.bytes_written > chunk.len()
            || result.bytes_written > remaining
        {
            return Err(GitPackError::ObjectSizeLimitExceeded);
        }
        input_offset += result.bytes_consumed;
        output.extend_from_slice(&chunk[..result.bytes_written]);

        match result.status {
            Ok(MZStatus::StreamEnd) => return Ok((output, input_offset)),
            Ok(MZStatus::Ok) => {
                if result.bytes_consumed == 0 && result.bytes_written == 0 {
                    return if input_offset == compressed.len() {
                        Err(GitPackError::DecompressionTruncated)
                    } else {
                        Err(GitPackError::DecompressionStalled)
                    };
                }
            }
            Ok(_) => return Err(GitPackError::DecompressionData),
            Err(MZError::Buf) if input_offset == compressed.len() => {
                return Err(GitPackError::DecompressionTruncated);
            }
            Err(MZError::Buf) => return Err(GitPackError::DecompressionStalled),
            Err(_) => return Err(GitPackError::DecompressionData),
        }
    }
}

fn apply_delta(base: &[u8], delta: &[u8], max_output: usize) -> Result<Vec<u8>, GitPackError> {
    let mut cursor = 0_usize;
    let source_size = read_delta_varint(delta, &mut cursor)?;
    if source_size != base.len() as u64 {
        return Err(GitPackError::DeltaBaseSizeMismatch);
    }
    let target_size = read_delta_varint(delta, &mut cursor)?;
    let target_size =
        usize::try_from(target_size).map_err(|_| GitPackError::ObjectSizeLimitExceeded)?;
    if target_size > max_output {
        return Err(GitPackError::ObjectSizeLimitExceeded);
    }

    let mut output = Vec::with_capacity(target_size);
    while cursor < delta.len() {
        let opcode = delta[cursor];
        cursor += 1;
        if opcode & 0x80 != 0 {
            let mut copy_offset = 0_u64;
            let mut copy_size = 0_u64;
            for bit in 0..4 {
                if opcode & (1 << bit) != 0 {
                    let byte = *delta
                        .get(cursor)
                        .ok_or(GitPackError::InvalidDeltaInstruction)?;
                    cursor += 1;
                    copy_offset |= u64::from(byte) << (bit * 8);
                }
            }
            for bit in 0..3 {
                if opcode & (1 << (4 + bit)) != 0 {
                    let byte = *delta
                        .get(cursor)
                        .ok_or(GitPackError::InvalidDeltaInstruction)?;
                    cursor += 1;
                    copy_size |= u64::from(byte) << (bit * 8);
                }
            }
            if copy_size == 0 {
                copy_size = 0x1_0000;
            }
            let start =
                usize::try_from(copy_offset).map_err(|_| GitPackError::InvalidDeltaInstruction)?;
            let count =
                usize::try_from(copy_size).map_err(|_| GitPackError::InvalidDeltaInstruction)?;
            let end = start
                .checked_add(count)
                .ok_or(GitPackError::InvalidDeltaInstruction)?;
            let source = base
                .get(start..end)
                .ok_or(GitPackError::InvalidDeltaInstruction)?;
            if output.len().saturating_add(source.len()) > target_size {
                return Err(GitPackError::DeltaTargetSizeMismatch);
            }
            output.extend_from_slice(source);
        } else if opcode != 0 {
            let count = usize::from(opcode);
            let end = cursor
                .checked_add(count)
                .ok_or(GitPackError::InvalidDeltaInstruction)?;
            let inserted = delta
                .get(cursor..end)
                .ok_or(GitPackError::InvalidDeltaInstruction)?;
            cursor = end;
            if output.len().saturating_add(inserted.len()) > target_size {
                return Err(GitPackError::DeltaTargetSizeMismatch);
            }
            output.extend_from_slice(inserted);
        } else {
            return Err(GitPackError::InvalidDeltaInstruction);
        }
    }
    if output.len() != target_size {
        return Err(GitPackError::DeltaTargetSizeMismatch);
    }
    Ok(output)
}

fn read_delta_varint(bytes: &[u8], cursor: &mut usize) -> Result<u64, GitPackError> {
    let mut value = 0_u64;
    let mut shift = 0_u32;
    for _ in 0..10 {
        let byte = *bytes.get(*cursor).ok_or(GitPackError::InvalidDeltaHeader)?;
        *cursor += 1;
        value |= u64::from(byte & 0x7f)
            .checked_shl(shift)
            .ok_or(GitPackError::InvalidDeltaHeader)?;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
    }
    Err(GitPackError::InvalidDeltaHeader)
}

fn canonical_object_id(kind: PackedObjectKind, body: &[u8]) -> Result<PackObjectId, GitPackError> {
    let header = format!("{} {}\0", kind.canonical_name(), body.len());
    let mut sha1 = GitObjectSha1::new();
    sha1.update(header.as_bytes())?;
    sha1.update(body)?;
    Ok(PackObjectId::from_bytes(sha1.finalize()?))
}

fn validate_fanout(fanout: &[u32; 256], object_ids: &[PackObjectId]) -> Result<(), GitPackError> {
    let mut counts = [0_u32; 256];
    for id in object_ids {
        let prefix = usize::from(id.bytes()[0]);
        counts[prefix] = counts[prefix]
            .checked_add(1)
            .ok_or(GitPackError::InvalidFanout)?;
    }
    let mut cumulative = 0_u32;
    for (index, count) in counts.into_iter().enumerate() {
        cumulative = cumulative
            .checked_add(count)
            .ok_or(GitPackError::InvalidFanout)?;
        if fanout[index] != cumulative {
            return Err(GitPackError::InvalidFanout);
        }
    }
    Ok(())
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[derive(Debug)]
pub enum GitPackError {
    InvalidBounds,
    IndexByteLimitExceeded,
    PackByteLimitExceeded,
    TruncatedIndex,
    InvalidIndexMagic,
    UnsupportedIndexVersion(u32),
    IndexChecksumMismatch,
    IndexTrailingData,
    InvalidFanout,
    ObjectLimitExceeded,
    ObjectIdsNotStrictlySorted,
    OffsetTableInvalid,
    TruncatedPack,
    InvalidPackMagic,
    UnsupportedPackVersion(u32),
    PackObjectCountMismatch,
    PackChecksumMismatch,
    PackIndexChecksumMismatch,
    PackOffsetInvalid,
    MissingPackedObject(PackObjectId),
    PackedEntryCrcMismatch,
    TruncatedPackEntry,
    InvalidPackObjectHeader,
    UnsupportedPackObjectType(u8),
    PackedRepresentationSizeMismatch,
    ObjectSizeLimitExceeded,
    InvalidOfsDelta,
    MissingDeltaBaseOffset(u64),
    ThinPackUnsupported(PackObjectId),
    DeltaDepthExceeded,
    DeltaCycle,
    Decompression(DecompressionBudgetError),
    DecompressionData,
    DecompressionTruncated,
    DecompressionStalled,
    InvalidDeltaHeader,
    InvalidDeltaInstruction,
    DeltaBaseSizeMismatch,
    DeltaTargetSizeMismatch,
    PackedObjectHashMismatch,
    Sha1(GitObjectSha1Error),
}

impl fmt::Display for GitPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBounds => {
                f.write_str("Git pack bounds exceed the frozen first-profile limits")
            }
            Self::IndexByteLimitExceeded => f.write_str("Git pack index byte limit exceeded"),
            Self::PackByteLimitExceeded => f.write_str("Git pack byte limit exceeded"),
            Self::TruncatedIndex => f.write_str("Git pack index is truncated"),
            Self::InvalidIndexMagic => f.write_str("Git pack index v2 magic is invalid"),
            Self::UnsupportedIndexVersion(version) => {
                write!(f, "unsupported Git pack index version: {version}")
            }
            Self::IndexChecksumMismatch => f.write_str("Git pack index SHA-1 checksum mismatch"),
            Self::IndexTrailingData => f.write_str("Git pack index contains trailing data"),
            Self::InvalidFanout => f.write_str("Git pack index fanout table is inconsistent"),
            Self::ObjectLimitExceeded => f.write_str("Git pack object limit exceeded"),
            Self::ObjectIdsNotStrictlySorted => {
                f.write_str("Git pack index object ids are not strictly sorted")
            }
            Self::OffsetTableInvalid => f.write_str("Git pack index offset table is invalid"),
            Self::TruncatedPack => f.write_str("Git pack file is truncated"),
            Self::InvalidPackMagic => f.write_str("Git pack magic is invalid"),
            Self::UnsupportedPackVersion(version) => {
                write!(f, "unsupported Git pack version: {version}")
            }
            Self::PackObjectCountMismatch => {
                f.write_str("Git pack object count does not match its index")
            }
            Self::PackChecksumMismatch => f.write_str("Git pack SHA-1 checksum mismatch"),
            Self::PackIndexChecksumMismatch => {
                f.write_str("Git pack checksum does not match pack index binding")
            }
            Self::PackOffsetInvalid => {
                f.write_str("Git pack index contains an invalid or duplicate object offset")
            }
            Self::MissingPackedObject(_) => {
                f.write_str("requested object is absent from this Git pack index")
            }
            Self::PackedEntryCrcMismatch => {
                f.write_str("Git packed object CRC32 does not match pack index")
            }
            Self::TruncatedPackEntry => f.write_str("Git packed object entry is truncated"),
            Self::InvalidPackObjectHeader => f.write_str("Git packed object header is invalid"),
            Self::UnsupportedPackObjectType(kind) => {
                write!(f, "unsupported Git packed object type: {kind}")
            }
            Self::PackedRepresentationSizeMismatch => {
                f.write_str("Git packed representation size does not match its header")
            }
            Self::ObjectSizeLimitExceeded => f.write_str("Git packed object size limit exceeded"),
            Self::InvalidOfsDelta => f.write_str("Git OFS_DELTA base distance is invalid"),
            Self::MissingDeltaBaseOffset(offset) => write!(
                f,
                "Git OFS_DELTA base offset is absent from pack index: {offset}"
            ),
            Self::ThinPackUnsupported(_) => f.write_str(
                "Git REF_DELTA refers outside the local pack; thin packs are unsupported",
            ),
            Self::DeltaDepthExceeded => f.write_str("Git delta chain depth limit exceeded"),
            Self::DeltaCycle => f.write_str("Git delta cycle detected"),
            Self::Decompression(error) => {
                write!(f, "bounded Git pack decompression failed: {error}")
            }
            Self::DecompressionData => f.write_str("Git packed object contains invalid zlib data"),
            Self::DecompressionTruncated => {
                f.write_str("Git packed object zlib stream is truncated")
            }
            Self::DecompressionStalled => {
                f.write_str("Git packed object decompression made no progress")
            }
            Self::InvalidDeltaHeader => f.write_str("Git delta header is invalid"),
            Self::InvalidDeltaInstruction => f.write_str("Git delta instruction is invalid"),
            Self::DeltaBaseSizeMismatch => {
                f.write_str("Git delta source size does not match reconstructed base")
            }
            Self::DeltaTargetSizeMismatch => {
                f.write_str("Git delta output size does not match declared target")
            }
            Self::PackedObjectHashMismatch => {
                f.write_str("reconstructed Git packed object SHA-1 does not match index object id")
            }
            Self::Sha1(error) => write!(f, "Git pack SHA-1 failed: {error}"),
        }
    }
}

impl Error for GitPackError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decompression(error) => Some(error),
            Self::Sha1(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DecompressionBudgetError> for GitPackError {
    fn from(value: DecompressionBudgetError) -> Self {
        Self::Decompression(value)
    }
}

impl From<GitObjectSha1Error> for GitPackError {
    fn from(value: GitObjectSha1Error) -> Self {
        Self::Sha1(value)
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.position)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], GitPackError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(GitPackError::TruncatedIndex)?;
        let output = self
            .bytes
            .get(self.position..end)
            .ok_or(GitPackError::TruncatedIndex)?;
        self.position = end;
        Ok(output)
    }

    fn read_u32(&mut self) -> Result<u32, GitPackError> {
        Ok(u32::from_be_bytes(
            self.take(4)?
                .try_into()
                .map_err(|_| GitPackError::TruncatedIndex)?,
        ))
    }

    fn read_u64(&mut self) -> Result<u64, GitPackError> {
        Ok(u64::from_be_bytes(
            self.take(8)?
                .try_into()
                .map_err(|_| GitPackError::TruncatedIndex)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v2_index_and_reads_base_blob() {
        let body = b"bounded pack object";
        let fixture = PackFixture::single_base(PackedObjectKind::Blob, body);
        let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
        let object = read_packed_object(
            &fixture.pack,
            &index,
            fixture.object_ids[0],
            GitPackBounds::default(),
        )
        .unwrap();
        assert_eq!(object.kind, PackedObjectKind::Blob);
        assert_eq!(object.bytes, body);
    }

    #[test]
    fn resolves_bounded_ofs_delta_and_verifies_reconstructed_identity() {
        let fixture = PackFixture::base_and_ofs_delta(b"hello", b"hello!");
        let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
        let object = read_packed_object(
            &fixture.pack,
            &index,
            fixture.object_ids[1],
            GitPackBounds::default(),
        )
        .unwrap();
        assert_eq!(object.kind, PackedObjectKind::Blob);
        assert_eq!(object.bytes, b"hello!");
    }

    #[test]
    fn rejects_index_checksum_fanout_and_pack_checksum_corruption() {
        let fixture = PackFixture::single_base(PackedObjectKind::Blob, b"x");

        let mut corrupt_index = fixture.index.clone();
        corrupt_index[20] ^= 1;
        assert!(matches!(
            parse_pack_index_v2(&corrupt_index, GitPackBounds::default()),
            Err(GitPackError::IndexChecksumMismatch)
        ));

        let mut bad_fanout = fixture.index.clone();
        bad_fanout[8 + 255 * 4..8 + 256 * 4].copy_from_slice(&0_u32.to_be_bytes());
        rewrite_index_checksum(&mut bad_fanout);
        assert!(matches!(
            parse_pack_index_v2(&bad_fanout, GitPackBounds::default()),
            Err(GitPackError::InvalidFanout)
        ));

        let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
        let mut corrupt_pack = fixture.pack.clone();
        corrupt_pack[12] ^= 1;
        assert!(matches!(
            read_packed_object(
                &corrupt_pack,
                &index,
                fixture.object_ids[0],
                GitPackBounds::default()
            ),
            Err(GitPackError::PackChecksumMismatch)
        ));
    }

    #[test]
    fn rejects_crc_mismatch_and_external_ref_delta_base() {
        let fixture = PackFixture::single_base(PackedObjectKind::Blob, b"crc");
        let mut bad_index = fixture.index.clone();
        let object_count = 1_usize;
        let crc_offset = INDEX_HEADER_BYTES + INDEX_FANOUT_BYTES + object_count * SHA1_BYTES;
        bad_index[crc_offset] ^= 1;
        rewrite_index_checksum(&mut bad_index);
        let index = parse_pack_index_v2(&bad_index, GitPackBounds::default()).unwrap();
        assert!(matches!(
            read_packed_object(
                &fixture.pack,
                &index,
                fixture.object_ids[0],
                GitPackBounds::default()
            ),
            Err(GitPackError::PackedEntryCrcMismatch)
        ));

        let thin = PackFixture::thin_ref_delta();
        let index = parse_pack_index_v2(&thin.index, GitPackBounds::default()).unwrap();
        assert!(matches!(
            read_packed_object(
                &thin.pack,
                &index,
                thin.object_ids[0],
                GitPackBounds::default()
            ),
            Err(GitPackError::ThinPackUnsupported(_))
        ));
    }

    #[test]
    fn rejects_unsupported_pack_and_index_versions_and_bounds() {
        let fixture = PackFixture::single_base(PackedObjectKind::Blob, b"version");
        let mut index_v3 = fixture.index.clone();
        index_v3[4..8].copy_from_slice(&3_u32.to_be_bytes());
        rewrite_index_checksum(&mut index_v3);
        assert!(matches!(
            parse_pack_index_v2(&index_v3, GitPackBounds::default()),
            Err(GitPackError::UnsupportedIndexVersion(3))
        ));

        let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
        let mut pack_v3 = fixture.pack.clone();
        pack_v3[4..8].copy_from_slice(&3_u32.to_be_bytes());
        rewrite_pack_checksum(&mut pack_v3);
        assert!(matches!(
            read_packed_object(
                &pack_v3,
                &index,
                fixture.object_ids[0],
                GitPackBounds::default()
            ),
            Err(GitPackError::UnsupportedPackVersion(3))
        ));

        let bounds = GitPackBounds {
            max_objects: 0,
            ..GitPackBounds::default()
        };
        assert!(matches!(
            bounds.validate(),
            Err(GitPackError::InvalidBounds)
        ));
    }

    struct PackFixture {
        pack: Vec<u8>,
        index: Vec<u8>,
        object_ids: Vec<PackObjectId>,
    }

    struct Record {
        id: PackObjectId,
        offset: u32,
        crc: u32,
    }

    impl PackFixture {
        fn single_base(kind: PackedObjectKind, body: &[u8]) -> Self {
            let mut pack = pack_header(1);
            let offset = pack.len() as u32;
            let mut entry = encode_pack_header(kind_type(kind), body.len());
            entry.extend_from_slice(&zlib_store(body));
            let crc = crc32(&entry);
            pack.extend_from_slice(&entry);
            let id = canonical_object_id(kind, body).unwrap();
            let pack_checksum = GitObjectSha1::digest(&pack).unwrap();
            pack.extend_from_slice(&pack_checksum);
            let index = build_index(&[Record { id, offset, crc }], pack_checksum);
            Self {
                pack,
                index,
                object_ids: vec![id],
            }
        }

        fn base_and_ofs_delta(base: &[u8], target: &[u8]) -> Self {
            assert_eq!(target, b"hello!");
            assert_eq!(base, b"hello");
            let mut pack = pack_header(2);

            let base_offset = pack.len() as u32;
            let mut base_entry = encode_pack_header(3, base.len());
            base_entry.extend_from_slice(&zlib_store(base));
            let base_crc = crc32(&base_entry);
            pack.extend_from_slice(&base_entry);

            let delta_offset = pack.len() as u32;
            let mut delta = vec![
                base.len() as u8,
                target.len() as u8,
                0x90,
                base.len() as u8,
                1,
                b'!',
            ];
            let mut delta_entry = encode_pack_header(6, delta.len());
            let distance = delta_offset - base_offset;
            assert!(distance < 128);
            delta_entry.push(distance as u8);
            delta_entry.extend_from_slice(&zlib_store(&delta));
            let delta_crc = crc32(&delta_entry);
            pack.extend_from_slice(&delta_entry);
            delta.clear();

            let base_id = canonical_object_id(PackedObjectKind::Blob, base).unwrap();
            let target_id = canonical_object_id(PackedObjectKind::Blob, target).unwrap();
            let pack_checksum = GitObjectSha1::digest(&pack).unwrap();
            pack.extend_from_slice(&pack_checksum);
            let index = build_index(
                &[
                    Record {
                        id: base_id,
                        offset: base_offset,
                        crc: base_crc,
                    },
                    Record {
                        id: target_id,
                        offset: delta_offset,
                        crc: delta_crc,
                    },
                ],
                pack_checksum,
            );
            Self {
                pack,
                index,
                object_ids: vec![base_id, target_id],
            }
        }

        fn thin_ref_delta() -> Self {
            let mut pack = pack_header(1);
            let offset = pack.len() as u32;
            let fake_base = PackObjectId::from_bytes([0x55; SHA1_BYTES]);
            let delta = [0_u8, 1_u8, 1_u8, b'x'];
            let mut entry = encode_pack_header(7, delta.len());
            entry.extend_from_slice(&fake_base.bytes());
            entry.extend_from_slice(&zlib_store(&delta));
            let crc = crc32(&entry);
            pack.extend_from_slice(&entry);
            let target_id = PackObjectId::from_bytes([0x77; SHA1_BYTES]);
            let pack_checksum = GitObjectSha1::digest(&pack).unwrap();
            pack.extend_from_slice(&pack_checksum);
            let index = build_index(
                &[Record {
                    id: target_id,
                    offset,
                    crc,
                }],
                pack_checksum,
            );
            Self {
                pack,
                index,
                object_ids: vec![target_id],
            }
        }
    }

    fn pack_header(count: u32) -> Vec<u8> {
        let mut pack = Vec::new();
        pack.extend_from_slice(b"PACK");
        pack.extend_from_slice(&2_u32.to_be_bytes());
        pack.extend_from_slice(&count.to_be_bytes());
        pack
    }

    fn kind_type(kind: PackedObjectKind) -> u8 {
        match kind {
            PackedObjectKind::Commit => 1,
            PackedObjectKind::Tree => 2,
            PackedObjectKind::Blob => 3,
            PackedObjectKind::Tag => 4,
        }
    }

    fn encode_pack_header(type_id: u8, mut size: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut first = (type_id & 0x07) << 4 | (size as u8 & 0x0f);
        size >>= 4;
        if size != 0 {
            first |= 0x80;
        }
        bytes.push(first);
        while size != 0 {
            let mut byte = (size as u8) & 0x7f;
            size >>= 7;
            if size != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
        }
        bytes
    }

    fn build_index(records: &[Record], pack_checksum: [u8; SHA1_BYTES]) -> Vec<u8> {
        let mut ordered = records
            .iter()
            .map(|record| Record {
                id: record.id,
                offset: record.offset,
                crc: record.crc,
            })
            .collect::<Vec<_>>();
        ordered.sort_by_key(|record| record.id);

        let mut counts = [0_u32; 256];
        for record in &ordered {
            counts[usize::from(record.id.bytes()[0])] += 1;
        }
        let mut cumulative = 0_u32;

        let mut index = Vec::new();
        index.extend_from_slice(&PACK_INDEX_MAGIC);
        index.extend_from_slice(&2_u32.to_be_bytes());
        for count in counts {
            cumulative += count;
            index.extend_from_slice(&cumulative.to_be_bytes());
        }
        for record in &ordered {
            index.extend_from_slice(&record.id.bytes());
        }
        for record in &ordered {
            index.extend_from_slice(&record.crc.to_be_bytes());
        }
        for record in &ordered {
            index.extend_from_slice(&record.offset.to_be_bytes());
        }
        index.extend_from_slice(&pack_checksum);
        let checksum = GitObjectSha1::digest(&index).unwrap();
        index.extend_from_slice(&checksum);
        index
    }

    fn rewrite_index_checksum(index: &mut [u8]) {
        let checksum_start = index.len() - SHA1_BYTES;
        let checksum = GitObjectSha1::digest(&index[..checksum_start]).unwrap();
        index[checksum_start..].copy_from_slice(&checksum);
    }

    fn rewrite_pack_checksum(pack: &mut [u8]) {
        let checksum_start = pack.len() - SHA1_BYTES;
        let checksum = GitObjectSha1::digest(&pack[..checksum_start]).unwrap();
        pack[checksum_start..].copy_from_slice(&checksum);
    }

    fn zlib_store(data: &[u8]) -> Vec<u8> {
        assert!(data.len() <= u16::MAX as usize);
        let len = data.len() as u16;
        let mut output = vec![0x78, 0x01, 0x01];
        output.extend_from_slice(&len.to_le_bytes());
        output.extend_from_slice(&(!len).to_le_bytes());
        output.extend_from_slice(data);
        output.extend_from_slice(&adler32(data).to_be_bytes());
        output
    }

    fn adler32(data: &[u8]) -> u32 {
        const MOD: u32 = 65_521;
        let mut a = 1_u32;
        let mut b = 0_u32;
        for byte in data {
            a = (a + u32::from(*byte)) % MOD;
            b = (b + a) % MOD;
        }
        b << 16 | a
    }
}
