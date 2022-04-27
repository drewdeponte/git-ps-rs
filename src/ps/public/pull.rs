use super::super::private::utils;
use super::super::private::git;

#[derive(Debug)]
pub enum PullError {
  RepositoryMissing,
  GetHeadBranchNameFailed,
  GetUpstreamBranchNameFailed,
  RebaseFailed(utils::ExecuteError),
  FetchFailed(git::ExtFetchError),
}

pub fn pull() -> Result<(), PullError> {
  let repo = git::create_cwd_repo().map_err(|_| PullError::RepositoryMissing)?;

  let head_ref = repo.head().map_err(|_| PullError::GetHeadBranchNameFailed)?;
  let head_branch_shorthand = head_ref.shorthand().ok_or(PullError::GetHeadBranchNameFailed)?;
  let head_branch_name = head_ref.name().ok_or(PullError::GetHeadBranchNameFailed)?;

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).map_err(|_| PullError::GetUpstreamBranchNameFailed)?;

  git::ext_fetch().map_err(PullError::FetchFailed)?;

  utils::execute("git", &["rebase", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]).map_err(PullError::RebaseFailed)
}
