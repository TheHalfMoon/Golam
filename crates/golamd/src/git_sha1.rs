#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

const SHA1_BLOCK_BYTES: usize = 64;
const SHA1_DIGEST_BYTES: usize = 20;
const SHA1_LENGTH_FIELD_BYTES: usize = 8;

/// SHA-1 state restricted to legacy Git object identity.
///
/// This type is not an authorization, signature, integrity-chain, or general
/// security primitive. It exists only to reproduce the object-id algorithm
/// required by the frozen SHA-1 Git repository profile in T005-040.
#[derive(Clone, Debug)]
pub struct GitObjectSha1 {
    state: [u32; 5],
    buffer: [u8; SHA1_BLOCK_BYTES],
    buffer_len: usize,
    total_len_bytes: u64,
}

impl Default for GitObjectSha1 {
    fn default() -> Self {
        Self::new()
    }
}

impl GitObjectSha1 {
    pub const fn new() -> Self {
        Self {
            state: [
                0x6745_2301,
                0xefcd_ab89,
                0x98ba_dcfe,
                0x1032_5476,
                0xc3d2_e1f0,
            ],
            buffer: [0; SHA1_BLOCK_BYTES],
            buffer_len: 0,
            total_len_bytes: 0,
        }
    }

    pub fn update(&mut self, mut input: &[u8]) -> Result<(), GitObjectSha1Error> {
        let input_len =
            u64::try_from(input.len()).map_err(|_| GitObjectSha1Error::MessageTooLong)?;
        self.total_len_bytes = self
            .total_len_bytes
            .checked_add(input_len)
            .ok_or(GitObjectSha1Error::MessageTooLong)?;

        if self.buffer_len != 0 {
            let needed = SHA1_BLOCK_BYTES - self.buffer_len;
            let take = needed.min(input.len());
            self.buffer[self.buffer_len..self.buffer_len + take].copy_from_slice(&input[..take]);
            self.buffer_len += take;
            input = &input[take..];

            if self.buffer_len == SHA1_BLOCK_BYTES {
                let block = self.buffer;
                self.compress_block(&block);
                self.buffer_len = 0;
            }
        }

        let mut chunks = input.chunks_exact(SHA1_BLOCK_BYTES);
        for chunk in &mut chunks {
            let block: &[u8; SHA1_BLOCK_BYTES] = chunk
                .try_into()
                .map_err(|_| GitObjectSha1Error::InternalBlockLength)?;
            self.compress_block(block);
        }
        let remainder = chunks.remainder();
        self.buffer[..remainder.len()].copy_from_slice(remainder);
        self.buffer_len = remainder.len();
        Ok(())
    }

    pub fn finalize(mut self) -> Result<[u8; SHA1_DIGEST_BYTES], GitObjectSha1Error> {
        let bit_len = self
            .total_len_bytes
            .checked_mul(8)
            .ok_or(GitObjectSha1Error::MessageTooLong)?;

        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > SHA1_BLOCK_BYTES - SHA1_LENGTH_FIELD_BYTES {
            self.buffer[self.buffer_len..].fill(0);
            let block = self.buffer;
            self.compress_block(&block);
            self.buffer = [0; SHA1_BLOCK_BYTES];
            self.buffer_len = 0;
        }

        self.buffer[self.buffer_len..SHA1_BLOCK_BYTES - SHA1_LENGTH_FIELD_BYTES].fill(0);
        self.buffer[SHA1_BLOCK_BYTES - SHA1_LENGTH_FIELD_BYTES..]
            .copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.compress_block(&block);

        let mut digest = [0_u8; SHA1_DIGEST_BYTES];
        for (index, word) in self.state.iter().enumerate() {
            let offset = index * 4;
            digest[offset..offset + 4].copy_from_slice(&word.to_be_bytes());
        }
        Ok(digest)
    }

    pub fn digest(input: &[u8]) -> Result<[u8; SHA1_DIGEST_BYTES], GitObjectSha1Error> {
        let mut hasher = Self::new();
        hasher.update(input)?;
        hasher.finalize()
    }

    fn compress_block(&mut self, block: &[u8; SHA1_BLOCK_BYTES]) {
        let mut schedule = [0_u32; 80];
        for (index, word) in block.chunks_exact(4).enumerate() {
            schedule[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..80 {
            schedule[index] = (schedule[index - 3]
                ^ schedule[index - 8]
                ^ schedule[index - 14]
                ^ schedule[index - 16])
                .rotate_left(1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        for (index, word) in schedule.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6),
            };
            let next = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = next;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitObjectSha1Error {
    MessageTooLong,
    InternalBlockLength,
}

impl fmt::Display for GitObjectSha1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MessageTooLong => {
                f.write_str("Git object SHA-1 input length exceeds the algorithm limit")
            }
            Self::InternalBlockLength => {
                f.write_str("Git object SHA-1 internal block length invariant failed")
            }
        }
    }
}

impl Error for GitObjectSha1Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_published_sha1_known_answer_vectors() {
        assert_eq!(
            hex(GitObjectSha1::digest(b"").unwrap()),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            hex(GitObjectSha1::digest(b"abc").unwrap()),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            hex(
                GitObjectSha1::digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",)
                    .unwrap()
            ),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
    }

    #[test]
    fn streaming_updates_match_one_shot_digest() {
        let bytes = b"bounded Git object identity must be stable across update boundaries";
        let expected = GitObjectSha1::digest(bytes).unwrap();
        let mut streaming = GitObjectSha1::new();
        for chunk in bytes.chunks(3) {
            streaming.update(chunk).unwrap();
        }
        assert_eq!(streaming.finalize().unwrap(), expected);
    }

    #[test]
    fn matches_empty_git_blob_object_identity() {
        assert_eq!(
            hex(GitObjectSha1::digest(b"blob 0\0").unwrap()),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
    }

    #[test]
    fn million_a_vector_matches_known_answer() {
        let chunk = [b'a'; 1_000];
        let mut hasher = GitObjectSha1::new();
        for _ in 0..1_000 {
            hasher.update(&chunk).unwrap();
        }
        assert_eq!(
            hex(hasher.finalize().unwrap()),
            "34aa973cd4c4daa4f61eeb2bdbad27316534016f"
        );
    }

    fn hex(bytes: [u8; SHA1_DIGEST_BYTES]) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(SHA1_DIGEST_BYTES * 2);
        for byte in bytes {
            output.push(char::from(DIGITS[(byte >> 4) as usize]));
            output.push(char::from(DIGITS[(byte & 0x0f) as usize]));
        }
        output
    }
}
