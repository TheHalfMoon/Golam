use core::{fmt, str::FromStr};

const CANONICAL_ID_HEX_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessIdParseError {
    InvalidLength,
    InvalidHex,
}

impl fmt::Display for HarnessIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => {
                f.write_str("harness identifier must be exactly 32 hexadecimal characters")
            }
            Self::InvalidHex => {
                f.write_str("harness identifier contains non-hexadecimal characters")
            }
        }
    }
}

impl std::error::Error for HarnessIdParseError {}

macro_rules! opaque_harness_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u128);

        impl $name {
            pub const fn from_u128(value: u128) -> Self {
                Self(value)
            }

            pub const fn as_u128(self) -> u128 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:032x}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = HarnessIdParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                if value.len() != CANONICAL_ID_HEX_LEN {
                    return Err(HarnessIdParseError::InvalidLength);
                }
                if !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
                    return Err(HarnessIdParseError::InvalidHex);
                }
                let parsed =
                    u128::from_str_radix(value, 16).map_err(|_| HarnessIdParseError::InvalidHex)?;
                Ok(Self(parsed))
            }
        }

        impl TryFrom<&str> for $name {
            type Error = HarnessIdParseError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                value.parse()
            }
        }
    };
}

opaque_harness_id!(ExecutionProfileId);
opaque_harness_id!(HardwareProfileId);
opaque_harness_id!(RequestSeriesId);
opaque_harness_id!(RequestAttemptId);
opaque_harness_id!(CompactionId);
opaque_harness_id!(ToolCallCandidateId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_have_stable_canonical_text() {
        let id = RequestAttemptId::from_u128(0x12ab);
        assert_eq!(id.to_string(), "000000000000000000000000000012ab");
        assert_eq!(id.to_string().parse::<RequestAttemptId>().unwrap(), id);
    }

    #[test]
    fn identifier_parsing_is_bounded() {
        assert_eq!(
            "12ab".parse::<ExecutionProfileId>(),
            Err(HarnessIdParseError::InvalidLength)
        );
        assert_eq!(
            "0000000000000000000000000000000z".parse::<ExecutionProfileId>(),
            Err(HarnessIdParseError::InvalidHex)
        );
        assert_eq!(
            "+0000000000000000000000000000001".parse::<ExecutionProfileId>(),
            Err(HarnessIdParseError::InvalidHex)
        );
        assert_eq!(
            "0000000000000000000000000000000A".parse::<ExecutionProfileId>(),
            Err(HarnessIdParseError::InvalidHex)
        );
    }

    #[test]
    fn every_spec_004_identifier_is_fixed_width() {
        assert_eq!(ExecutionProfileId::from_u128(1).to_string().len(), 32);
        assert_eq!(HardwareProfileId::from_u128(1).to_string().len(), 32);
        assert_eq!(RequestSeriesId::from_u128(1).to_string().len(), 32);
        assert_eq!(RequestAttemptId::from_u128(1).to_string().len(), 32);
        assert_eq!(CompactionId::from_u128(1).to_string().len(), 32);
        assert_eq!(ToolCallCandidateId::from_u128(1).to_string().len(), 32);
    }
}
