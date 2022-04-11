use super::super::super::ps;

#[derive(Debug)]
pub enum RequestReviewError {
  SyncFailed(ps::public::sync::SyncError)
}

pub fn rr(patch_index: usize) -> Result<(), RequestReviewError> {
  ps::public::sync::sync(patch_index).map_err(|e| RequestReviewError::SyncFailed(e))?;
  Ok(())
}
