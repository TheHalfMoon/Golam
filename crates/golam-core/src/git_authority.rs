#![forbid(unsafe_code)]

use core::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OrdinaryGitMutation {
    Add,
    Commit,
    BranchCreate,
}

impl OrdinaryGitMutation {
    pub const fn action(self) -> &'static str {
        match self {
            Self::Add => "git.add",
            Self::Commit => "git.commit",
            Self::BranchCreate => "git.branch.create",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DestructiveGitOperation {
    ForcePush,
    ForceRefMove,
    BranchOverwrite,
    Rebase,
    SharedHistoryRewrite,
}

impl DestructiveGitOperation {
    pub const fn action(self) -> &'static str {
        match self {
            Self::ForcePush => "git.force-push",
            Self::ForceRefMove => "git.force-ref-move",
            Self::BranchOverwrite => "git.branch.overwrite",
            Self::Rebase => "git.rebase",
            Self::SharedHistoryRewrite => "git.history.rewrite",
        }
    }

    pub const fn ordinary_authority_disposition(self) -> GitAuthorityDisposition {
        let _ = self;
        GitAuthorityDisposition::UnavailableOutsideOrdinaryAuthority
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GitAuthorityDisposition {
    OrdinaryMutation,
    UnavailableOutsideOrdinaryAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitAuthorityError {
    DestructiveOperationUnavailable,
    UnsupportedOperation,
}

impl fmt::Display for GitAuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DestructiveOperationUnavailable => f.write_str(
                "destructive Git operation is unavailable outside explicit non-ordinary authority",
            ),
            Self::UnsupportedOperation => {
                f.write_str("Git operation is outside the bounded ordinary mutation vocabulary")
            }
        }
    }
}

impl std::error::Error for GitAuthorityError {}

pub fn classify_ordinary_git_action(
    action: &str,
) -> Result<OrdinaryGitMutation, GitAuthorityError> {
    match action {
        "git.add" => Ok(OrdinaryGitMutation::Add),
        "git.commit" => Ok(OrdinaryGitMutation::Commit),
        "git.branch.create" => Ok(OrdinaryGitMutation::BranchCreate),
        "git.force-push"
        | "git.force-ref-move"
        | "git.branch.overwrite"
        | "git.rebase"
        | "git.history.rewrite" => Err(GitAuthorityError::DestructiveOperationUnavailable),
        _ => Err(GitAuthorityError::UnsupportedOperation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_git_mutation_vocabulary_is_exact_and_bounded() {
        for operation in [
            OrdinaryGitMutation::Add,
            OrdinaryGitMutation::Commit,
            OrdinaryGitMutation::BranchCreate,
        ] {
            assert_eq!(classify_ordinary_git_action(operation.action()), Ok(operation));
        }
        assert_eq!(
            classify_ordinary_git_action("git.push"),
            Err(GitAuthorityError::UnsupportedOperation)
        );
        assert_eq!(
            classify_ordinary_git_action("git.reset.hard"),
            Err(GitAuthorityError::UnsupportedOperation)
        );
    }

    #[test]
    fn every_destructive_git_operation_is_explicitly_unavailable() {
        for operation in [
            DestructiveGitOperation::ForcePush,
            DestructiveGitOperation::ForceRefMove,
            DestructiveGitOperation::BranchOverwrite,
            DestructiveGitOperation::Rebase,
            DestructiveGitOperation::SharedHistoryRewrite,
        ] {
            assert_eq!(
                operation.ordinary_authority_disposition(),
                GitAuthorityDisposition::UnavailableOutsideOrdinaryAuthority
            );
            assert_eq!(
                classify_ordinary_git_action(operation.action()),
                Err(GitAuthorityError::DestructiveOperationUnavailable)
            );
        }
    }
}
