use std::result::Result;
use super::super::private::utils;

#[derive(Debug)]
pub enum AmendPatchError {
  CommitFailed(utils::ExecuteError)
}

pub fn amend_patch(no_edit: bool) -> Result<(), AmendPatchError>  {
  let args = if no_edit {
    vec!["commit", "--amend", "--no-edit"]
  } else {
    vec!["commit", "--amend"]
  };
  utils::execute("git", &args).map_err(AmendPatchError::CommitFailed)?;
  Ok(())
}
