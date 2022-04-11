use super::super::private::utils;
use super::super::private::git;

#[derive(Debug)]
pub enum RebaseError {
  RepositoryMissing,
  GetHeadBranchNameFailed,
  GetUpstreamBranchNameFailed,
  RebaseFailed(utils::ExecuteError)
}

pub fn rebase() -> Result<(), RebaseError> {
  let repo = git::create_cwd_repo().map_err(|_| RebaseError::RepositoryMissing)?;

  let head_ref = repo.head().map_err(|_| RebaseError::GetHeadBranchNameFailed)?;
  let head_branch_shorthand = head_ref.shorthand().ok_or(RebaseError::GetHeadBranchNameFailed)?;
  let head_branch_name = head_ref.name().ok_or(RebaseError::GetHeadBranchNameFailed)?;

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).map_err(|_| RebaseError::GetUpstreamBranchNameFailed)?;

  utils::execute("git", &["rebase", "-i", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]).map_err(|e| RebaseError::RebaseFailed(e))
}
