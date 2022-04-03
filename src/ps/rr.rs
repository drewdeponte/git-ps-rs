use super::git;
use super::super::ps;

#[derive(Debug)]
pub enum RequestReviewError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  CreateRrBranchFailed(ps::plumbing::branch::BranchError),
  RequestReviewBranchNameMissing,
  ForcePushFailed(ps::plumbing::git::ExtForcePushError)
}

pub fn rr(patch_index: usize) -> Result<(), RequestReviewError> {
  let repo = git::create_cwd_repo().map_err(|_| RequestReviewError::RepositoryNotFound)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(RequestReviewError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| RequestReviewError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| RequestReviewError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  let branch = ps::plumbing::branch::branch(&repo, patch_index).map_err(|e| RequestReviewError::CreateRrBranchFailed(e))?;

  // force push request review branch up to remote
  let branch_ref_name = branch.get().name().ok_or(RequestReviewError::RequestReviewBranchNameMissing)?;
  git::ext_force_push(remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).map_err(|e| RequestReviewError::ForcePushFailed(e))
}
