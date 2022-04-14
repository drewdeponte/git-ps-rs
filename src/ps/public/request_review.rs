use super::super::super::ps;

#[derive(Debug)]
pub enum RequestReviewError {
  SyncFailed(ps::public::sync::SyncError)
}

pub fn request_review(patch_index: usize, given_branch_name: Option<String>) -> Result<(), RequestReviewError> {
  ps::public::sync::sync(patch_index, given_branch_name).map_err(RequestReviewError::SyncFailed)?;
  Ok(())
}
