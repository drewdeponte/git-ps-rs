use super::super::private;
use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum BranchError {
    OpenRepositoryFailed(Box<dyn std::error::Error>),
    ConflictsExist(String, String),
    MergeCommitDetected(String),
    Unhandled(Box<dyn std::error::Error>),
}

impl From<private::branch::BranchError> for BranchError {
    fn from(value: private::branch::BranchError) -> Self {
        match value {
            private::branch::BranchError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            private::branch::BranchError::MergeCommitDetected(oid) => {
                Self::MergeCommitDetected(oid)
            }
            _ => Self::Unhandled(value.into()),
        }
    }
}

impl std::fmt::Display for BranchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenRepositoryFailed(e) => write!(f, "failed to open repository {}", e),
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict(s) found when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::MergeCommitDetected(oid) => write!(f, "merge commit detected with sha {}", oid),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BranchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenRepositoryFailed(e) => Some(e.as_ref()),
            Self::ConflictsExist(_, _) => None,
            Self::MergeCommitDetected(_) => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn branch(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    branch_name: Option<String>,
) -> Result<(), BranchError> {
    let repo = git::create_cwd_repo().map_err(|e| BranchError::OpenRepositoryFailed(e.into()))?;
    private::branch::branch(&repo, start_patch_index, end_patch_index, branch_name)?;
    Ok(())
}
