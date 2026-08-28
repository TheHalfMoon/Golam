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
/// ordering cannot affect the canonical representation. Union only adds bits;
/// downgrade/removal is deliberately not part of this type.
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

    pub const fn contains_all(self, required: Self) -> bool {
        self.bits & required.bits == required.bits
    }

    /// Monotonic provenance composition. There is intentionally no inverse or
    /// removal operation; downgrade authority belongs to later attested paths.
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
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

    /// Strict inverse of `canonical_bytes` for protected-state reads.
    ///
    /// Non-canonical order, duplicate labels, unknown codes, wrong domain,
    /// trailing bytes, and impossible counts fail closed rather than being
    /// normalized on read.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, CoreError> {
        let prefix_len = 4_usize
            .checked_add(TAINT_SET_DOMAIN.len())
            .and_then(|value| value.checked_add(1))
            .ok_or(CoreError::InvalidCanonicalTaintSet)?;
        if bytes.len() < prefix_len {
            return Err(CoreError::InvalidCanonicalTaintSet);
        }

        let domain_len = u32::from_be_bytes(
            bytes[..4]
                .try_into()
                .map_err(|_| CoreError::InvalidCanonicalTaintSet)?,
        );
        if usize::try_from(domain_len).map_err(|_| CoreError::InvalidCanonicalTaintSet)?
            != TAINT_SET_DOMAIN.len()
            || &bytes[4..4 + TAINT_SET_DOMAIN.len()] != TAINT_SET_DOMAIN
        {
            return Err(CoreError::InvalidCanonicalTaintSet);
        }

        let count_index = 4 + TAINT_SET_DOMAIN.len();
        let count = bytes[count_index];
        if count > MAX_TAINT_LABELS || bytes.len() != prefix_len + usize::from(count) {
            return Err(CoreError::InvalidCanonicalTaintSet);
        }

        let mut set = Self::empty();
        let mut previous_code = 0_u8;
        for code in &bytes[prefix_len..] {
            if *code <= previous_code {
                return Err(CoreError::InvalidCanonicalTaintSet);
            }
            let label = TaintLabel::from_code(*code).ok_or(CoreError::InvalidCanonicalTaintSet)?;
            set.bits |= bit(label);
            previous_code = *code;
        }
        if set.len() != count {
            return Err(CoreError::InvalidCanonicalTaintSet);
        }
        Ok(set)
    }
}

/// Carries provenance beside a value without changing the value's own identity
/// or canonical bytes. This is suitable for artifact receipts and authority
/// context values whose domain identity must remain independent of provenance.
///
/// `derive` is the only constructor for derived values: it unions every source
/// set with labels introduced by the transform. The wrapper exposes no taint
/// mutation or removal method.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenanced<T> {
    value: T,
    taint: TaintSet,
}

impl<T> Provenanced<T> {
    /// Establishes provenance at an explicit source/trust boundary.
    pub const fn source(value: T, taint: TaintSet) -> Self {
        Self { value, taint }
    }

    /// Creates a derived value whose provenance is the union of all source
    /// labels plus labels introduced by the transform itself.
    pub fn derive(
        value: T,
        source_taint: impl IntoIterator<Item = TaintSet>,
        introduced: TaintSet,
    ) -> Self {
        let taint = source_taint.into_iter().fold(introduced, TaintSet::union);
        Self { value, taint }
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn taint(&self) -> TaintSet {
        self.taint
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
        assert_eq!(
            first.canonical_bytes().unwrap(),
            second.canonical_bytes().unwrap()
        );

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

    #[test]
    fn canonical_decode_round_trips_and_rejects_noncanonical_bytes() {
        let set = TaintSet::from_labels([
            TaintLabel::WebUntrusted,
            TaintLabel::ModelGenerated,
            TaintLabel::SecretDerived,
        ]);
        let bytes = set.canonical_bytes().unwrap();
        assert_eq!(TaintSet::from_canonical_bytes(&bytes).unwrap(), set);

        let mut duplicate = bytes.clone();
        let last = *duplicate.last().unwrap();
        *duplicate.get_mut(4 + TAINT_SET_DOMAIN.len()).unwrap() = 4;
        duplicate.push(last);
        assert_eq!(
            TaintSet::from_canonical_bytes(&duplicate),
            Err(CoreError::InvalidCanonicalTaintSet)
        );

        let mut out_of_order = bytes.clone();
        let labels_start = 5 + TAINT_SET_DOMAIN.len();
        out_of_order.swap(labels_start, labels_start + 1);
        assert_eq!(
            TaintSet::from_canonical_bytes(&out_of_order),
            Err(CoreError::InvalidCanonicalTaintSet)
        );

        let mut trailing = bytes;
        trailing.push(0xff);
        assert_eq!(
            TaintSet::from_canonical_bytes(&trailing),
            Err(CoreError::InvalidCanonicalTaintSet)
        );
    }

    #[test]
    fn union_is_monotonic_commutative_and_idempotent() {
        let web = TaintSet::from_labels([TaintLabel::WebUntrusted]);
        let model = TaintSet::from_labels([TaintLabel::ModelGenerated]);
        let combined = web.union(model);

        assert!(combined.contains_all(web));
        assert!(combined.contains_all(model));
        assert_eq!(combined, model.union(web));
        assert_eq!(combined, combined.union(web));
    }

    #[test]
    fn derived_value_cannot_lose_source_or_transform_labels() {
        let web = TaintSet::from_labels([TaintLabel::WebUntrusted]);
        let mcp = TaintSet::from_labels([TaintLabel::McpUntrusted]);
        let generated = TaintSet::from_labels([TaintLabel::ModelGenerated]);
        let derived = Provenanced::derive("summary", [mcp, web], generated);

        assert_eq!(derived.value(), &"summary");
        assert!(derived.taint().contains_all(web));
        assert!(derived.taint().contains_all(mcp));
        assert!(derived.taint().contains_all(generated));

        let reordered = Provenanced::derive("summary", [web, mcp], generated);
        assert_eq!(derived.taint(), reordered.taint());
        assert_eq!(
            derived.taint().canonical_bytes().unwrap(),
            reordered.taint().canonical_bytes().unwrap()
        );
    }
}
