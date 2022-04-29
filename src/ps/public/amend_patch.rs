use std::result::Result;
use super::super::private::utils;

#[derive(Debug)]
pub enum AmendPatchError {
  CommitFailed(utils::ExecuteError)
}

pub fn amend_patch() -> Result<(), AmendPatchError>  {
  utils::execute("git", &["commit", "-v", "--amend"]).map_err(AmendPatchError::CommitFailed)?;
  Ok(())
}
