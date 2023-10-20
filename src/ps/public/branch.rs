use super::super::private;
use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum BranchError {
    OpenRepositoryFailed(git::CreateCwdRepositoryError),
    BranchOperationFailed(private::branch::BranchError),
}

pub fn branch(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    branch_name: Option<String>,
) -> Result<(), BranchError> {
    let repo = git::create_cwd_repo().map_err(BranchError::OpenRepositoryFailed)?;
    private::branch::branch(&repo, start_patch_index, end_patch_index, branch_name)
        .map_err(BranchError::BranchOperationFailed)?;
    Ok(())
}
