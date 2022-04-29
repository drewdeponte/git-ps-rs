use std::result::Result;
use super::super::private::utils;

#[derive(Debug)]
pub enum CreatePatchError {
  CommitFailed(utils::ExecuteError)
}

pub fn create_patch() -> Result<(), CreatePatchError>  {
  utils::execute("git", &["commit", "-v"]).map_err(CreatePatchError::CommitFailed)?;
  Ok(())
}
