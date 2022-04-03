use super::git;
use super::super::ps;

#[derive(Debug)]
pub enum IntegrateError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  CreateRrBranchFailed(ps::plumbing::branch::BranchError),
  RequestReviewBranchNameMissing,
  ForcePushFailed(ps::plumbing::git::ExtForcePushError),
  PushFailed(ps::plumbing::git::ExtForcePushError)
}

pub fn integrate(patch_index: usize) -> Result<(), IntegrateError> {
  let repo = git::create_cwd_repo().map_err(|_| IntegrateError::RepositoryNotFound)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| IntegrateError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  let branch = ps::plumbing::branch::branch(&repo, patch_index).map_err(|e| IntegrateError::CreateRrBranchFailed(e))?;

  // force push request review branch up to remote
  let branch_ref_name = branch.get().name().ok_or(IntegrateError::RequestReviewBranchNameMissing)?;
  git::ext_push(true, remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).map_err(|e| IntegrateError::ForcePushFailed(e))?;

  // - push rr branch up to upstream branch (e.g. origin/main)
  let pattern = format!("refs/remotes/{}/", remote_name.as_str().unwrap());
  let remote_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");

  git::ext_push(false, remote_name.as_str().unwrap(), branch_ref_name, &remote_branch_shorthand).map_err(|e| IntegrateError::PushFailed(e))?;

  Ok(())
}
