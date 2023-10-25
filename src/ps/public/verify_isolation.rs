use super::isolate;
use super::isolate::IsolateError;
use std::result::Result;

#[derive(Debug)]
pub enum VerifyIsolationError {
    MergeCommitDetected(String),
    ConflictsExist(String, String),
    UncommittedChangesExist,
    IsolateFailed(IsolateError),
    IsolateResetFailed(IsolateError),
}

fn isolate_failed_err_map(e: IsolateError) -> VerifyIsolationError {
    match e {
        IsolateError::MergeCommitDetected(oid) => VerifyIsolationError::MergeCommitDetected(oid),
        IsolateError::ConflictsExist(src_oid, dst_oid) => {
            VerifyIsolationError::ConflictsExist(src_oid, dst_oid)
        }
        IsolateError::UncommittedChangesExist => VerifyIsolationError::UncommittedChangesExist,
        _ => VerifyIsolationError::IsolateFailed(e),
    }
}

fn isolate_reset_failed_err_map(e: IsolateError) -> VerifyIsolationError {
    match e {
        IsolateError::MergeCommitDetected(oid) => VerifyIsolationError::MergeCommitDetected(oid),
        IsolateError::ConflictsExist(src_oid, dst_oid) => {
            VerifyIsolationError::ConflictsExist(src_oid, dst_oid)
        }
        IsolateError::UncommittedChangesExist => VerifyIsolationError::UncommittedChangesExist,
        _ => VerifyIsolationError::IsolateResetFailed(e),
    }
}

impl std::fmt::Display for VerifyIsolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict detected when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::MergeCommitDetected(oid) => write!(f, "merge commit detected with sha {}", oid),
            Self::UncommittedChangesExist => write!(f, "uncommitted changes exist"),
            Self::IsolateResetFailed(e) => {
                write!(f, "failed to reset back to previous branch, {}", e)
            }
            Self::IsolateFailed(e) => write!(f, "failed to isolate, {}", e),
        }
    }
}

impl std::error::Error for VerifyIsolationError {}

pub fn verify_isolation(
    patch_index: usize,
    end_patch_index_optional: Option<usize>,
    color: bool,
) -> Result<(), VerifyIsolationError> {
    match isolate::isolate(Some(patch_index), end_patch_index_optional, color) {
        Ok(_) => Ok(isolate::isolate(None, None, color)
            .map_err(VerifyIsolationError::IsolateResetFailed)?),
        Err(e) => match e {
            // pre-checkout errors
            IsolateError::OpenGitRepositoryFailed(_)
            | IsolateError::OpenGitConfigFailed(_)
            | IsolateError::UncommittedChangesExistFailure(_)
            | IsolateError::UncommittedChangesExist
            | IsolateError::GetPatchStackFailed(_)
            | IsolateError::PatchStackBaseNotFound
            | IsolateError::GetPatchListFailed(_)
            | IsolateError::PatchIndexNotFound
            | IsolateError::CreateBranchFailed
            | IsolateError::BranchNameNotUtf8
            | IsolateError::MergeCommitDetected(_)
            | IsolateError::ConflictsExist(_, _)
            | IsolateError::GetCurrentBranchFailed
            | IsolateError::StoreLastBranchFailed(_)
            | IsolateError::FailedToCheckout(_) => Err(isolate_failed_err_map(e)),
            // post-checkout errors
            _ => {
                isolate::isolate(None, None, color).map_err(isolate_reset_failed_err_map)?;
                Err(isolate_failed_err_map(e))
            }
        },
    }
}
