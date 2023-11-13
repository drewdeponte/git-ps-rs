use super::commit_diff::{commit_diff, CommitDiffError};
use git2;
use std::result::Result;

#[derive(Debug)]
pub enum CommitDiffPatchIdError {
    GetDiffFailed(CommitDiffError),
    CreatePatchHashFailed(git2::Error),
}

impl std::fmt::Display for CommitDiffPatchIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetDiffFailed(e) => write!(f, "get diff failed, {}", e),
            Self::CreatePatchHashFailed(e) => write!(f, "create patch hash failed, {}", e),
        }
    }
}

impl std::error::Error for CommitDiffPatchIdError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GetDiffFailed(e) => Some(e),
            Self::CreatePatchHashFailed(e) => Some(e),
        }
    }
}

pub fn commit_diff_patch_id(
    repo: &git2::Repository,
    commit: &git2::Commit,
) -> Result<git2::Oid, CommitDiffPatchIdError> {
    let diff = commit_diff(repo, commit).map_err(CommitDiffPatchIdError::GetDiffFailed)?;
    diff.patchid(Option::None)
        .map_err(CommitDiffPatchIdError::CreatePatchHashFailed)
}
