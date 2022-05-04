use super::super::public::isolate;
use super::super::public::isolate::IsolateError;
use std::result::Result;

#[derive(Debug)]
pub enum VerifyIsolationError {
  IsolateFailed(IsolateError),
  IsolateResetFailed(IsolateError)
}

pub fn verify_isolation(patch_index: usize, color: bool) -> Result<(), VerifyIsolationError> {
  match isolate::isolate(Some(patch_index), color) {
    Ok(_) => Ok(isolate::isolate(None, color).map_err(VerifyIsolationError::IsolateResetFailed)?),
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
        | IsolateError::CherryPickFailed
        | IsolateError::GetCurrentBranchFailed
        | IsolateError::GetIsolateLastBranchPathFailed(_)
        | IsolateError::StoreLastBranchFailed(_)
        | IsolateError::FailedToCheckout(_) => Err(VerifyIsolationError::IsolateFailed(e)),
      // post-checkout errors
      _ => {
        isolate::isolate(None, color).map_err(VerifyIsolationError::IsolateResetFailed)?;
        Err(VerifyIsolationError::IsolateFailed(e))
      }
    }
  }
}
