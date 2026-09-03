#![forbid(unsafe_code)]

use crate::git_pack::{
    GitPackBounds, PackObjectId, PackedObjectKind, parse_pack_index_v2, read_packed_object,
};
use crate::git_sha1::GitObjectSha1;

const SHA1_BYTES: usize = 20;
const PACK_INDEX_MAGIC: [u8; 4] = [0xff, b't', b'O', b'c'];
const INDEX_HEADER_BYTES: usize = 8;
const INDEX_FANOUT_BYTES: usize = 256 * 4;

#[test]
fn reconstructs_valid_in_pack_ref_delta_and_verifies_target_identity() {
    let fixture = RefDeltaFixture::new(b"hello", b"hello!");
    let index = parse_pack_index_v2(&fixture.index, GitPackBounds::default()).unwrap();
    let object = read_packed_object(
        &fixture.pack,
        &index,
        fixture.target_id,
        GitPackBounds::default(),
    )
    .unwrap();

    assert_eq!(object.kind, PackedObjectKind::Blob);
    assert_eq!(object.id, fixture.target_id);
    assert_eq!(object.bytes, b"hello!");
}

struct RefDeltaFixture {
    pack: Vec<u8>,
    index: Vec<u8>,
    target_id: PackObjectId,
}

#[derive(Clone, Copy)]
struct Record {
    id: PackObjectId,
    offset: u32,
    crc: u32,
}

impl RefDeltaFixture {
    fn new(base: &[u8], target: &[u8]) -> Self {
        assert_eq!(base, b"hello");
        assert_eq!(target, b"hello!");

        let mut pack = pack_header(2);

        let base_offset = pack.len() as u32;
        let mut base_entry = encode_pack_header(3, base.len());
        base_entry.extend_from_slice(&zlib_store(base));
        let base_crc = crc32(&base_entry);
        pack.extend_from_slice(&base_entry);
        let base_id = canonical_blob_id(base);

        let target_offset = pack.len() as u32;
        let delta = [
            base.len() as u8,
            target.len() as u8,
            0x90,
            base.len() as u8,
            1,
            b'!',
        ];
        let mut target_entry = encode_pack_header(7, delta.len());
        target_entry.extend_from_slice(&base_id.bytes());
        target_entry.extend_from_slice(&zlib_store(&delta));
        let target_crc = crc32(&target_entry);
        pack.extend_from_slice(&target_entry);
        let target_id = canonical_blob_id(target);

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
                    offset: target_offset,
                    crc: target_crc,
                },
            ],
            pack_checksum,
        );

        Self {
            pack,
            index,
            target_id,
        }
    }
}

fn canonical_blob_id(body: &[u8]) -> PackObjectId {
    let header = format!("blob {}\0", body.len());
    let mut sha1 = GitObjectSha1::new();
    sha1.update(header.as_bytes()).unwrap();
    sha1.update(body).unwrap();
    PackObjectId::from_bytes(sha1.finalize().unwrap())
}

fn pack_header(count: u32) -> Vec<u8> {
    let mut pack = Vec::new();
    pack.extend_from_slice(b"PACK");
    pack.extend_from_slice(&2_u32.to_be_bytes());
    pack.extend_from_slice(&count.to_be_bytes());
    pack
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
    let mut ordered = records.to_vec();
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

    let expected_minimum = INDEX_HEADER_BYTES + INDEX_FANOUT_BYTES + SHA1_BYTES * 2;
    assert!(index.len() >= expected_minimum);
    index
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
