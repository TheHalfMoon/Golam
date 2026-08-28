#![forbid(unsafe_code)]

use crate::{CanonicalEncoder, CoreError};

const TAINT_SET_DOMAIN: &[u8] = b"golam:taint-label-set:v1";
const MAX_TAINT_LABELS: u8 = 9;

/// Closed baseline provenance labels frozen by Spec 003.
///
/// The numeric codes are part of the canonical encoding contract and must not
/// be renumbered. Adding a future label requires a versioned compatibility
/// decision rather than reusing an existing code.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TaintLabel {
    UserTrusted = 1,
    LocalTrusted = 2,
    LocalUnverified = 3,
    WebUntrusted = 4,
    ChannelUntrusted = 5,
    McpUntrusted = 6,
    PluginUnverified = 7,
    ModelGenerated = 8,
    SecretDerived = 9,
}

impl TaintLabel {
    pub const ALL: [Self; 9] = [
        Self::UserTrusted,
        Self::LocalTrusted,
        Self::LocalUnverified,
        Self::WebUntrusted,
        Self::ChannelUntrusted,
        Self::McpUntrusted,
        Self::PluginUnverified,
        Self::ModelGenerated,
        Self::SecretDerived,
    ];

    pub const fn code(self) -> u8 {
        self as u8
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserTrusted => "USER_TRUSTED",
            Self::LocalTrusted => "LOCAL_TRUSTED",
            Self::LocalUnverified => "LOCAL_UNVERIFIED",
            Self::WebUntrusted => "WEB_UNTRUSTED",
            Self::ChannelUntrusted => "CHANNEL_UNTRUSTED",
            Self::McpUntrusted => "MCP_UNTRUSTED",
            Self::PluginUnverified => "PLUGIN_UNVERIFIED",
            Self::ModelGenerated => "MODEL_GENERATED",
            Self::SecretDerived => "SECRET_DERIVED",
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::UserTrusted),
            2 => Some(Self::LocalTrusted),
            3 => Some(Self::LocalUnverified),
            4 => Some(Self::WebUntrusted),
            5 => Some(Self::ChannelUntrusted),
            6 => Some(Self::McpUntrusted),
            7 => Some(Self::PluginUnverified),
            8 => Some(Self::ModelGenerated),
            9 => Some(Self::SecretDerived),
            _ => None,
        }
    }
}

/// Canonical set of provenance labels.
///
/// Internally this is a bounded bitset so duplicate labels and caller-provided
/// ordering cannot affect the canonical representation. T003-040 deliberately
/// defines only representation; propagation/union policy is owned by T003-041.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TaintSet {
    bits: u16,
}

impl TaintSet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn from_labels(labels: impl IntoIterator<Item = TaintLabel>) -> Self {
        let mut set = Self::empty();
        for label in labels {
            set.bits |= bit(label);
        }
        set
    }

    pub const fn contains(self, label: TaintLabel) -> bool {
        self.bits & bit(label) != 0
    }

    pub const fn len(self) -> u8 {
        self.bits.count_ones() as u8
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn labels(self) -> impl Iterator<Item = TaintLabel> {
        TaintLabel::ALL
            .into_iter()
            .filter(move |label| self.contains(*label))
    }

    pub fn canonical_bytes(self) -> Result<Vec<u8>, CoreError> {
        let mut encoder = CanonicalEncoder::new();
        encoder.push_bytes(TAINT_SET_DOMAIN)?;
        encoder.push_u8(self.len());
        for label in self.labels() {
            encoder.push_u8(label.code());
        }
        Ok(encoder.finish())
    }
}

const fn bit(label: TaintLabel) -> u16 {
    1_u16 << (label.code() - 1)
}

const _: () = assert!(TaintLabel::ALL.len() == MAX_TAINT_LABELS as usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_labels_have_frozen_names_and_codes() {
        let expected = [
            (1, "USER_TRUSTED"),
            (2, "LOCAL_TRUSTED"),
            (3, "LOCAL_UNVERIFIED"),
            (4, "WEB_UNTRUSTED"),
            (5, "CHANNEL_UNTRUSTED"),
            (6, "MCP_UNTRUSTED"),
            (7, "PLUGIN_UNVERIFIED"),
            (8, "MODEL_GENERATED"),
            (9, "SECRET_DERIVED"),
        ];

        assert_eq!(TaintLabel::ALL.len(), expected.len());
        for (label, (code, name)) in TaintLabel::ALL.into_iter().zip(expected) {
            assert_eq!(label.code(), code);
            assert_eq!(label.as_str(), name);
            assert_eq!(TaintLabel::from_code(code), Some(label));
        }
        assert_eq!(TaintLabel::from_code(0), None);
        assert_eq!(TaintLabel::from_code(10), None);
    }

    #[test]
    fn set_deduplicates_and_orders_by_frozen_label_code() {
        let set = TaintSet::from_labels([
            TaintLabel::SecretDerived,
            TaintLabel::WebUntrusted,
            TaintLabel::SecretDerived,
            TaintLabel::UserTrusted,
        ]);

        assert_eq!(set.len(), 3);
        assert_eq!(
            set.labels().collect::<Vec<_>>(),
            vec![
                TaintLabel::UserTrusted,
                TaintLabel::WebUntrusted,
                TaintLabel::SecretDerived,
            ]
        );
    }

    #[test]
    fn canonical_encoding_is_order_and_duplicate_invariant() {
        let first = TaintSet::from_labels([
            TaintLabel::McpUntrusted,
            TaintLabel::ModelGenerated,
            TaintLabel::WebUntrusted,
        ]);
        let second = TaintSet::from_labels([
            TaintLabel::WebUntrusted,
            TaintLabel::McpUntrusted,
            TaintLabel::WebUntrusted,
            TaintLabel::ModelGenerated,
        ]);

        assert_eq!(first, second);
        assert_eq!(first.canonical_bytes().unwrap(), second.canonical_bytes().unwrap());

        let mut expected = Vec::new();
        expected.extend_from_slice(&(TAINT_SET_DOMAIN.len() as u32).to_be_bytes());
        expected.extend_from_slice(TAINT_SET_DOMAIN);
        expected.push(3);
        expected.extend_from_slice(&[
            TaintLabel::WebUntrusted.code(),
            TaintLabel::McpUntrusted.code(),
            TaintLabel::ModelGenerated.code(),
        ]);
        assert_eq!(first.canonical_bytes().unwrap(), expected);
    }

    #[test]
    fn empty_set_has_explicit_canonical_representation() {
        let encoded = TaintSet::empty().canonical_bytes().unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&(TAINT_SET_DOMAIN.len() as u32).to_be_bytes());
        expected.extend_from_slice(TAINT_SET_DOMAIN);
        expected.push(0);
        assert_eq!(encoded, expected);
    }

    #[test]
    fn full_set_is_bounded_to_the_spec_baseline() {
        let set = TaintSet::from_labels(TaintLabel::ALL);
        assert_eq!(set.len(), MAX_TAINT_LABELS);
        assert!(TaintLabel::ALL.into_iter().all(|label| set.contains(label)));
    }
}
