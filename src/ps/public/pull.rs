use super::super::private::utils;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::config;
use super::super::public::list;

#[derive(Debug)]
pub enum PullError {
  RepositoryMissing,
  GetHeadBranchNameFailed,
  GetUpstreamBranchNameFailed,
  RebaseFailed(utils::ExecuteError),
  FetchFailed(git::ExtFetchError),
  GetRepoRootPathFailed(paths::PathsError),
  PathNotUtf8,
  GetConfigFailed(config::GetConfigError),
  ListFailed(list::ListError)
}

pub fn pull() -> Result<(), PullError> {
  let repo = git::create_cwd_repo().map_err(|_| PullError::RepositoryMissing)?;

  let repo_root_path = paths::repo_root_path(&repo).map_err(PullError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(PullError::PathNotUtf8)?;
  let config = config::get_config(repo_root_str).map_err(PullError::GetConfigFailed)?;

  let head_ref = repo.head().map_err(|_| PullError::GetHeadBranchNameFailed)?;
  let head_branch_shorthand = head_ref.shorthand().ok_or(PullError::GetHeadBranchNameFailed)?;
  let head_branch_name = head_ref.name().ok_or(PullError::GetHeadBranchNameFailed)?;

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).map_err(|_| PullError::GetUpstreamBranchNameFailed)?;

  git::ext_fetch().map_err(PullError::FetchFailed)?;

  utils::execute("git", &["rebase", "--no-reapply-cherry-picks", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]).map_err(PullError::RebaseFailed)?;

  if config.pull.show_list_post_pull {
    list::list().map_err(PullError::ListFailed)?
  }

  Ok(())
}
