use super::super::private;
use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum RequestReviewBranchError {
    OpenRepositoryFailed(git::CreateCwdRepositoryError),
    BranchOperationFailed(private::request_review_branch::RequestReviewBranchError),
}

pub fn request_review_branch(
    patch_index: usize,
    branch_name: Option<String>,
) -> Result<(), RequestReviewBranchError> {
    let repo = git::create_cwd_repo().map_err(RequestReviewBranchError::OpenRepositoryFailed)?;
    private::request_review_branch::request_review_branch(&repo, patch_index, branch_name)
        .map_err(RequestReviewBranchError::BranchOperationFailed)?;
    Ok(())
}
