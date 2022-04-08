use super::super::ps;

#[derive(Debug)]
pub enum RequestReviewError {
  SyncFailed(ps::plumbing::sync::SyncError)
}

pub fn rr(patch_index: usize) -> Result<(), RequestReviewError> {
  ps::plumbing::sync::sync(patch_index).map_err(|e| RequestReviewError::SyncFailed(e))?;
  Ok(())
}
