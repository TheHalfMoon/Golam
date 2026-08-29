#![forbid(unsafe_code)]

use crate::egress_destination::EffectiveDestination;
use crate::egress_permit::EgressPermitRecord;

const EFFECTIVE_USE_CONTEXT_DOMAIN: &[u8] = b"golam:egress-effective-use-context:v2";

/// Runtime information-flow context that must exactly match the protected permit.
///
/// The context contains only a taint digest and an optional opaque secret-handle identifier.
/// It never carries secret plaintext.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EgressUseContext {
    taint_digest: [u8; 32],
    secret_handle_id: Option<[u8; 16]>,
}

impl EgressUseContext {
    pub const fn new(taint_digest: [u8; 32], secret_handle_id: Option<[u8; 16]>) -> Self {
        Self {
            taint_digest,
            secret_handle_id,
        }
    }

    pub const fn taint_digest(self) -> [u8; 32] {
        self.taint_digest
    }

    pub const fn secret_handle_id(self) -> Option<[u8; 16]> {
        self.secret_handle_id
    }

    pub(crate) fn matches_permit(self, permit: &EgressPermitRecord) -> bool {
        self.taint_digest == permit.taint_digest && self.secret_handle_id == permit.secret_handle_id
    }

    pub(crate) fn decision_context_hash(
        self,
        effective: &EffectiveDestination,
        permit_id: [u8; 16],
        authorized_destination: &str,
    ) -> [u8; 32] {
        let endpoint_hash = effective.decision_context_hash(permit_id, authorized_destination);
        let mut hasher = blake3::Hasher::new();
        hasher.update(EFFECTIVE_USE_CONTEXT_DOMAIN);
        hasher.update(&endpoint_hash);
        hasher.update(&self.taint_digest);
        match self.secret_handle_id {
            Some(handle_id) => {
                hasher.update(&[1]);
                hasher.update(&handle_id);
            }
            None => {
                hasher.update(&[0]);
            }
        }
        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint() -> EffectiveDestination {
        EffectiveDestination::new(
            "example.invalid",
            "203.0.113.10".parse().unwrap(),
            "https",
            443,
        )
        .unwrap()
    }

    #[test]
    fn decision_context_is_taint_and_secret_handle_sensitive() {
        let endpoint = endpoint();
        let permit_id = [9; 16];
        let destination = "https://example.invalid";
        let baseline = EgressUseContext::new([1; 32], None)
            .decision_context_hash(&endpoint, permit_id, destination);
        let changed_taint = EgressUseContext::new([2; 32], None)
            .decision_context_hash(&endpoint, permit_id, destination);
        let changed_handle = EgressUseContext::new([1; 32], Some([3; 16]))
            .decision_context_hash(&endpoint, permit_id, destination);

        assert_ne!(baseline, changed_taint);
        assert_ne!(baseline, changed_handle);
        assert_ne!(changed_taint, changed_handle);
        assert_eq!(
            baseline,
            EgressUseContext::new([1; 32], None)
                .decision_context_hash(&endpoint, permit_id, destination)
        );
    }
}
