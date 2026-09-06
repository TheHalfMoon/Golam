#![forbid(unsafe_code)]

use core::fmt;
use std::collections::HashSet;

use crate::tool_request::{BindingDigest, RequestedOperationId, RequestedTarget, ResourceClassId};

const MAX_ALLOWED_OPERATIONS: usize = 64;
const MAX_ALIAS_CHAIN_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlatformFamily {
    Unix,
    Windows,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedRoot {
    pub platform: PlatformFamily,
    pub policy_resource_class: ResourceClassId,
    pub resolved_root_identity: BindingDigest,
    pub allowed_operations: Vec<RequestedOperationId>,
}

impl AuthorizedRoot {
    pub fn validate(&self) -> Result<(), TargetIdentityError> {
        if self.allowed_operations.is_empty() {
            return Err(TargetIdentityError::MissingAllowedOperation);
        }
        if self.allowed_operations.len() > MAX_ALLOWED_OPERATIONS {
            return Err(TargetIdentityError::TooManyAllowedOperations);
        }
        if self
            .allowed_operations
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(TargetIdentityError::NonCanonicalAllowedOperations);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObservedFileKind {
    Missing,
    RegularFile,
    Directory,
    SymlinkOrReparsePoint,
    Special,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedTargetIdentity {
    pub platform: PlatformFamily,
    pub requested_path: RequestedTarget,
    pub normalized_path: RequestedTarget,
    pub resolved_parent_identity: Option<BindingDigest>,
    pub resolved_target_identity: Option<BindingDigest>,
    pub file_kind: ObservedFileKind,
    pub symlink_or_reparse_chain: Vec<BindingDigest>,
    pub observed_metadata_digest: BindingDigest,
    pub observed_at_unix_ms: u64,
}

impl ResolvedTargetIdentity {
    pub fn validate(&self) -> Result<(), TargetIdentityError> {
        if self.symlink_or_reparse_chain.len() > MAX_ALIAS_CHAIN_DEPTH {
            return Err(TargetIdentityError::AliasChainTooDeep);
        }
        let mut seen = HashSet::with_capacity(self.symlink_or_reparse_chain.len());
        if self
            .symlink_or_reparse_chain
            .iter()
            .any(|identity| !seen.insert(*identity))
        {
            return Err(TargetIdentityError::AliasCycleOrDuplicate);
        }

        match self.file_kind {
            ObservedFileKind::Missing => {
                if self.resolved_target_identity.is_some() {
                    return Err(TargetIdentityError::MissingTargetHasIdentity);
                }
                if self.resolved_parent_identity.is_none() {
                    return Err(TargetIdentityError::MissingTargetNeedsParentIdentity);
                }
            }
            _ => {
                if self.resolved_target_identity.is_none() {
                    return Err(TargetIdentityError::ExistingTargetNeedsIdentity);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileMutationExpectation {
    pub expected_exists: bool,
    pub expected_kind: Option<ObservedFileKind>,
    pub expected_identity: Option<BindingDigest>,
    pub expected_content_digest: Option<BindingDigest>,
    pub expected_size: Option<u64>,
    pub expected_parent_identity: Option<BindingDigest>,
}

impl FileMutationExpectation {
    pub fn validate(self) -> Result<(), TargetIdentityError> {
        if self.expected_exists {
            if self.expected_kind.is_none() || self.expected_identity.is_none() {
                return Err(TargetIdentityError::ExistingExpectationNeedsIdentityAndKind);
            }
        } else if self.expected_kind.is_some()
            || self.expected_identity.is_some()
            || self.expected_content_digest.is_some()
            || self.expected_size.is_some()
        {
            return Err(TargetIdentityError::MissingExpectationHasTargetState);
        }

        if !self.expected_exists && self.expected_parent_identity.is_none() {
            return Err(TargetIdentityError::CreateExpectationNeedsParentIdentity);
        }

        if (self.expected_content_digest.is_some() || self.expected_size.is_some())
            && self.expected_kind != Some(ObservedFileKind::RegularFile)
        {
            return Err(TargetIdentityError::ContentExpectationRequiresRegularFile);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetIdentityError {
    MissingAllowedOperation,
    TooManyAllowedOperations,
    NonCanonicalAllowedOperations,
    AliasChainTooDeep,
    AliasCycleOrDuplicate,
    MissingTargetHasIdentity,
    MissingTargetNeedsParentIdentity,
    ExistingTargetNeedsIdentity,
    ExistingExpectationNeedsIdentityAndKind,
    MissingExpectationHasTargetState,
    CreateExpectationNeedsParentIdentity,
    ContentExpectationRequiresRegularFile,
}

impl fmt::Display for TargetIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAllowedOperation => {
                f.write_str("authorized root has no allowed operation")
            }
            Self::TooManyAllowedOperations => {
                f.write_str("authorized root exceeds bounded operation count")
            }
            Self::NonCanonicalAllowedOperations => {
                f.write_str("authorized root operations must be strictly sorted and unique")
            }
            Self::AliasChainTooDeep => f.write_str("target alias chain exceeds bounded depth"),
            Self::AliasCycleOrDuplicate => {
                f.write_str("target alias chain contains a cycle or duplicate identity")
            }
            Self::MissingTargetHasIdentity => {
                f.write_str("missing target cannot have a resolved target identity")
            }
            Self::MissingTargetNeedsParentIdentity => {
                f.write_str("missing target requires a resolved parent identity")
            }
            Self::ExistingTargetNeedsIdentity => {
                f.write_str("existing target requires a resolved target identity")
            }
            Self::ExistingExpectationNeedsIdentityAndKind => {
                f.write_str("existing mutation expectation requires target identity and kind")
            }
            Self::MissingExpectationHasTargetState => {
                f.write_str("nonexistent-target expectation cannot bind target state")
            }
            Self::CreateExpectationNeedsParentIdentity => {
                f.write_str("nonexistent-target expectation requires expected parent identity")
            }
            Self::ContentExpectationRequiresRegularFile => {
                f.write_str("content or size expectation requires a regular file")
            }
        }
    }
}

impl std::error::Error for TargetIdentityError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: u8) -> BindingDigest {
        BindingDigest::new([value; 32])
    }

    #[test]
    fn authorized_root_requires_canonical_bounded_operations() {
        let root = AuthorizedRoot {
            platform: PlatformFamily::Unix,
            policy_resource_class: ResourceClassId::new("workspace.fs").unwrap(),
            resolved_root_identity: digest(1),
            allowed_operations: vec![
                RequestedOperationId::new("list").unwrap(),
                RequestedOperationId::new("read").unwrap(),
            ],
        };
        assert_eq!(root.validate(), Ok(()));

        let mut invalid = root;
        invalid.allowed_operations.reverse();
        assert_eq!(
            invalid.validate(),
            Err(TargetIdentityError::NonCanonicalAllowedOperations)
        );
    }

    #[test]
    fn missing_target_requires_parent_and_no_target_identity() {
        let identity = ResolvedTargetIdentity {
            platform: PlatformFamily::Unix,
            requested_path: RequestedTarget::new("new.txt").unwrap(),
            normalized_path: RequestedTarget::new("/workspace/new.txt").unwrap(),
            resolved_parent_identity: Some(digest(1)),
            resolved_target_identity: None,
            file_kind: ObservedFileKind::Missing,
            symlink_or_reparse_chain: vec![],
            observed_metadata_digest: digest(2),
            observed_at_unix_ms: 10,
        };
        assert_eq!(identity.validate(), Ok(()));

        let mut invalid = identity;
        invalid.resolved_target_identity = Some(digest(3));
        assert_eq!(
            invalid.validate(),
            Err(TargetIdentityError::MissingTargetHasIdentity)
        );
    }

    #[test]
    fn alias_chain_rejects_duplicate_identity() {
        let identity = ResolvedTargetIdentity {
            platform: PlatformFamily::Windows,
            requested_path: RequestedTarget::new("src\\lib.rs").unwrap(),
            normalized_path: RequestedTarget::new("C:\\workspace\\src\\lib.rs").unwrap(),
            resolved_parent_identity: Some(digest(1)),
            resolved_target_identity: Some(digest(2)),
            file_kind: ObservedFileKind::RegularFile,
            symlink_or_reparse_chain: vec![digest(3), digest(3)],
            observed_metadata_digest: digest(4),
            observed_at_unix_ms: 10,
        };
        assert_eq!(
            identity.validate(),
            Err(TargetIdentityError::AliasCycleOrDuplicate)
        );
    }

    #[test]
    fn mutation_expectation_is_fail_closed_for_inconsistent_state() {
        let create = FileMutationExpectation {
            expected_exists: false,
            expected_kind: None,
            expected_identity: None,
            expected_content_digest: None,
            expected_size: None,
            expected_parent_identity: Some(digest(1)),
        };
        assert_eq!(create.validate(), Ok(()));

        let stale = FileMutationExpectation {
            expected_exists: true,
            expected_kind: Some(ObservedFileKind::Directory),
            expected_identity: Some(digest(2)),
            expected_content_digest: Some(digest(3)),
            expected_size: None,
            expected_parent_identity: Some(digest(1)),
        };
        assert_eq!(
            stale.validate(),
            Err(TargetIdentityError::ContentExpectationRequiresRegularFile)
        );
    }
}
