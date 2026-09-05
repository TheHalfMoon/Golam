#![forbid(unsafe_code)]

//! Narrow executable-image admission for the first governed Spec 005 process profile.
//!
//! T005-078 deliberately does not admit scripts, dynamic ELF images, cross-architecture images,
//! or unbounded executable inputs. This parser is data validation only; it grants no authority and
//! does not execute a payload.

use std::error::Error;
use std::fmt;

pub const MAX_STATIC_EXECUTABLE_BYTES: usize = 32 * 1024 * 1024;
const ELF64_HEADER_BYTES: usize = 64;
const ELF64_PROGRAM_HEADER_BYTES: usize = 56;
const MAX_PROGRAM_HEADERS: usize = 128;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;
const EV_CURRENT: u32 = 1;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const PF_X: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StaticElfV2Evidence {
    pub byte_len: usize,
    pub program_header_count: u16,
    pub load_segment_count: u16,
    pub executable_load_segment_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaticElfV2Error {
    Empty,
    TooLarge,
    TruncatedHeader,
    InvalidMagic,
    UnsupportedClass,
    UnsupportedEndian,
    UnsupportedIdentVersion,
    UnsupportedOsAbi,
    UnsupportedElfType,
    UnsupportedMachine,
    UnsupportedVersion,
    InvalidHeaderSize,
    InvalidProgramHeaderShape,
    ProgramHeaderTableOutOfBounds,
    TooManyProgramHeaders,
    SegmentOutOfBounds,
    InvalidLoadSegment,
    DynamicSegmentForbidden,
    InterpreterForbidden,
    MissingLoadSegment,
    MissingExecutableLoadSegment,
}

impl fmt::Display for StaticElfV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "static ELF payload is empty",
            Self::TooLarge => "static ELF payload exceeds the bounded executable size",
            Self::TruncatedHeader => "static ELF header is truncated",
            Self::InvalidMagic => "payload is not an ELF image",
            Self::UnsupportedClass => "first process profile requires ELF64",
            Self::UnsupportedEndian => "first process profile requires little-endian ELF",
            Self::UnsupportedIdentVersion => "ELF identification version is unsupported",
            Self::UnsupportedOsAbi => "first process profile requires System V/Linux-neutral ELF ABI",
            Self::UnsupportedElfType => "first process profile requires a fixed ET_EXEC image",
            Self::UnsupportedMachine => "first process profile requires x86_64 ELF",
            Self::UnsupportedVersion => "ELF header version is unsupported",
            Self::InvalidHeaderSize => "ELF header size does not match ELF64",
            Self::InvalidProgramHeaderShape => "ELF program-header table shape is invalid",
            Self::ProgramHeaderTableOutOfBounds => "ELF program-header table is out of bounds",
            Self::TooManyProgramHeaders => "ELF program-header count exceeds the admission bound",
            Self::SegmentOutOfBounds => "ELF segment file range is out of bounds",
            Self::InvalidLoadSegment => "ELF load segment has invalid file/memory sizes",
            Self::DynamicSegmentForbidden => "dynamic ELF metadata is not admitted by the first process profile",
            Self::InterpreterForbidden => "PT_INTERP is not admitted by the first process profile",
            Self::MissingLoadSegment => "ELF image has no loadable segment",
            Self::MissingExecutableLoadSegment => "ELF image has no executable loadable segment",
        };
        f.write_str(message)
    }
}

impl Error for StaticElfV2Error {}

pub fn validate_static_elf_v2(bytes: &[u8]) -> Result<StaticElfV2Evidence, StaticElfV2Error> {
    if bytes.is_empty() {
        return Err(StaticElfV2Error::Empty);
    }
    if bytes.len() > MAX_STATIC_EXECUTABLE_BYTES {
        return Err(StaticElfV2Error::TooLarge);
    }
    if bytes.len() < ELF64_HEADER_BYTES {
        return Err(StaticElfV2Error::TruncatedHeader);
    }
    if &bytes[0..4] != b"\x7fELF" {
        return Err(StaticElfV2Error::InvalidMagic);
    }
    if bytes[4] != 2 {
        return Err(StaticElfV2Error::UnsupportedClass);
    }
    if bytes[5] != 1 {
        return Err(StaticElfV2Error::UnsupportedEndian);
    }
    if bytes[6] != 1 {
        return Err(StaticElfV2Error::UnsupportedIdentVersion);
    }
    // ELFOSABI_SYSV (0) and ELFOSABI_LINUX (3) do not imply extra runtime authority.
    if !matches!(bytes[7], 0 | 3) {
        return Err(StaticElfV2Error::UnsupportedOsAbi);
    }
    if read_u16(bytes, 16)? != ET_EXEC {
        return Err(StaticElfV2Error::UnsupportedElfType);
    }
    if read_u16(bytes, 18)? != EM_X86_64 {
        return Err(StaticElfV2Error::UnsupportedMachine);
    }
    if read_u32(bytes, 20)? != EV_CURRENT {
        return Err(StaticElfV2Error::UnsupportedVersion);
    }
    if usize::from(read_u16(bytes, 52)?) != ELF64_HEADER_BYTES {
        return Err(StaticElfV2Error::InvalidHeaderSize);
    }

    let program_header_offset = usize::try_from(read_u64(bytes, 32)?)
        .map_err(|_| StaticElfV2Error::ProgramHeaderTableOutOfBounds)?;
    let program_header_size = usize::from(read_u16(bytes, 54)?);
    let program_header_count = usize::from(read_u16(bytes, 56)?);
    if program_header_size != ELF64_PROGRAM_HEADER_BYTES || program_header_count == 0 {
        return Err(StaticElfV2Error::InvalidProgramHeaderShape);
    }
    if program_header_count > MAX_PROGRAM_HEADERS {
        return Err(StaticElfV2Error::TooManyProgramHeaders);
    }
    let program_header_bytes = program_header_size
        .checked_mul(program_header_count)
        .ok_or(StaticElfV2Error::ProgramHeaderTableOutOfBounds)?;
    let table_end = program_header_offset
        .checked_add(program_header_bytes)
        .ok_or(StaticElfV2Error::ProgramHeaderTableOutOfBounds)?;
    if program_header_offset < ELF64_HEADER_BYTES || table_end > bytes.len() {
        return Err(StaticElfV2Error::ProgramHeaderTableOutOfBounds);
    }

    let mut load_segment_count = 0_u16;
    let mut executable_load_segment_count = 0_u16;
    for index in 0..program_header_count {
        let offset = program_header_offset + index * program_header_size;
        let header = &bytes[offset..offset + program_header_size];
        let segment_type = read_u32(header, 0)?;
        if segment_type == PT_INTERP {
            return Err(StaticElfV2Error::InterpreterForbidden);
        }
        if segment_type == PT_DYNAMIC {
            return Err(StaticElfV2Error::DynamicSegmentForbidden);
        }

        let file_offset = usize::try_from(read_u64(header, 8)?)
            .map_err(|_| StaticElfV2Error::SegmentOutOfBounds)?;
        let file_size = usize::try_from(read_u64(header, 32)?)
            .map_err(|_| StaticElfV2Error::SegmentOutOfBounds)?;
        let memory_size = usize::try_from(read_u64(header, 40)?)
            .map_err(|_| StaticElfV2Error::InvalidLoadSegment)?;
        if file_size != 0 {
            let file_end = file_offset
                .checked_add(file_size)
                .ok_or(StaticElfV2Error::SegmentOutOfBounds)?;
            if file_end > bytes.len() {
                return Err(StaticElfV2Error::SegmentOutOfBounds);
            }
        }
        if segment_type == PT_LOAD {
            if file_size > memory_size {
                return Err(StaticElfV2Error::InvalidLoadSegment);
            }
            load_segment_count = load_segment_count
                .checked_add(1)
                .ok_or(StaticElfV2Error::TooManyProgramHeaders)?;
            if read_u32(header, 4)? & PF_X != 0 {
                executable_load_segment_count = executable_load_segment_count
                    .checked_add(1)
                    .ok_or(StaticElfV2Error::TooManyProgramHeaders)?;
            }
        }
    }
    if load_segment_count == 0 {
        return Err(StaticElfV2Error::MissingLoadSegment);
    }
    if executable_load_segment_count == 0 {
        return Err(StaticElfV2Error::MissingExecutableLoadSegment);
    }

    Ok(StaticElfV2Evidence {
        byte_len: bytes.len(),
        program_header_count: u16::try_from(program_header_count)
            .map_err(|_| StaticElfV2Error::TooManyProgramHeaders)?,
        load_segment_count,
        executable_load_segment_count,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, StaticElfV2Error> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or(StaticElfV2Error::TruncatedHeader)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, StaticElfV2Error> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or(StaticElfV2Error::TruncatedHeader)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, StaticElfV2Error> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or(StaticElfV2Error::TruncatedHeader)?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(segment_type: u32, segment_flags: u32) -> Vec<u8> {
        let mut bytes = vec![0_u8; ELF64_HEADER_BYTES + ELF64_PROGRAM_HEADER_BYTES + 16];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[6] = 1;
        bytes[7] = 0;
        bytes[16..18].copy_from_slice(&ET_EXEC.to_le_bytes());
        bytes[18..20].copy_from_slice(&EM_X86_64.to_le_bytes());
        bytes[20..24].copy_from_slice(&EV_CURRENT.to_le_bytes());
        bytes[32..40].copy_from_slice(&(ELF64_HEADER_BYTES as u64).to_le_bytes());
        bytes[52..54].copy_from_slice(&(ELF64_HEADER_BYTES as u16).to_le_bytes());
        bytes[54..56].copy_from_slice(&(ELF64_PROGRAM_HEADER_BYTES as u16).to_le_bytes());
        bytes[56..58].copy_from_slice(&1_u16.to_le_bytes());

        let ph = ELF64_HEADER_BYTES;
        bytes[ph..ph + 4].copy_from_slice(&segment_type.to_le_bytes());
        bytes[ph + 4..ph + 8].copy_from_slice(&segment_flags.to_le_bytes());
        bytes[ph + 8..ph + 16].copy_from_slice(
            &((ELF64_HEADER_BYTES + ELF64_PROGRAM_HEADER_BYTES) as u64).to_le_bytes(),
        );
        bytes[ph + 32..ph + 40].copy_from_slice(&16_u64.to_le_bytes());
        bytes[ph + 40..ph + 48].copy_from_slice(&16_u64.to_le_bytes());
        bytes
    }

    #[test]
    fn admits_bounded_static_x86_64_exec() {
        let evidence = validate_static_elf_v2(&image(PT_LOAD, PF_X)).unwrap();
        assert_eq!(evidence.program_header_count, 1);
        assert_eq!(evidence.load_segment_count, 1);
        assert_eq!(evidence.executable_load_segment_count, 1);
    }

    #[test]
    fn rejects_interpreter_and_dynamic_metadata() {
        assert_eq!(
            validate_static_elf_v2(&image(PT_INTERP, 0)),
            Err(StaticElfV2Error::InterpreterForbidden)
        );
        assert_eq!(
            validate_static_elf_v2(&image(PT_DYNAMIC, 0)),
            Err(StaticElfV2Error::DynamicSegmentForbidden)
        );
    }

    #[test]
    fn rejects_wrong_architecture_and_dynamic_type() {
        let mut wrong_machine = image(PT_LOAD, PF_X);
        wrong_machine[18..20].copy_from_slice(&3_u16.to_le_bytes());
        assert_eq!(
            validate_static_elf_v2(&wrong_machine),
            Err(StaticElfV2Error::UnsupportedMachine)
        );

        let mut pie = image(PT_LOAD, PF_X);
        pie[16..18].copy_from_slice(&3_u16.to_le_bytes());
        assert_eq!(
            validate_static_elf_v2(&pie),
            Err(StaticElfV2Error::UnsupportedElfType)
        );
    }

    #[test]
    fn rejects_out_of_bounds_program_headers_and_segments() {
        let mut bad_table = image(PT_LOAD, PF_X);
        bad_table[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            validate_static_elf_v2(&bad_table),
            Err(StaticElfV2Error::ProgramHeaderTableOutOfBounds)
        );

        let mut bad_segment = image(PT_LOAD, PF_X);
        let ph = ELF64_HEADER_BYTES;
        bad_segment[ph + 8..ph + 16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert_eq!(
            validate_static_elf_v2(&bad_segment),
            Err(StaticElfV2Error::SegmentOutOfBounds)
        );
    }

    #[test]
    fn requires_an_executable_load_segment() {
        assert_eq!(
            validate_static_elf_v2(&image(PT_LOAD, 0)),
            Err(StaticElfV2Error::MissingExecutableLoadSegment)
        );
    }
}
