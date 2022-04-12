use super::super::private::git;
use super::super::private;
use std::result::Result;

#[derive(Debug)]
pub enum BranchError {
  OpenRepositoryFailed(git::CreateCwdRepositoryError),
  BranchOperationFailed(private::branch::BranchError)
}

pub fn branch(patch_index: usize, branch_name: Option<String>) -> Result<(), BranchError>  {
  let repo = git::create_cwd_repo().map_err(BranchError::OpenRepositoryFailed)?;
  private::branch::branch(&repo, patch_index, branch_name).map_err(BranchError::BranchOperationFailed)?;
  Ok(())
}
