use super::super::private::utils;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::config;
use super::super::public::list;
use super::super::public::fetch;

#[derive(Debug)]
pub enum PullError {
  RepositoryMissing,
  GetHeadBranchNameFailed,
  GetUpstreamBranchNameFailed,
  RebaseFailed(utils::ExecuteError),
  FetchFailed(fetch::FetchError),
  GetRepoRootPathFailed(paths::PathsError),
  PathNotUtf8,
  GetConfigFailed(config::GetConfigError),
  ListFailed(list::ListError)
}

pub fn pull(color: bool) -> Result<(), PullError> {
  let repo = git::create_cwd_repo().map_err(|_| PullError::RepositoryMissing)?;

  let repo_root_path = paths::repo_root_path(&repo).map_err(PullError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(PullError::PathNotUtf8)?;
  let config = config::get_config(repo_root_str).map_err(PullError::GetConfigFailed)?;

  let head_ref = repo.head().map_err(|_| PullError::GetHeadBranchNameFailed)?;
  let head_branch_shorthand = head_ref.shorthand().ok_or(PullError::GetHeadBranchNameFailed)?;
  let head_branch_name = head_ref.name().ok_or(PullError::GetHeadBranchNameFailed)?;

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).map_err(|_| PullError::GetUpstreamBranchNameFailed)?;

  println!("Fetching upstream patches...");
  fetch::fetch(color).map_err(PullError::FetchFailed)?;
  println!();

  println!("Rebasing...");
  utils::execute("git", &["rebase", "--no-reapply-cherry-picks", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]).map_err(PullError::RebaseFailed)?;
  println!();

  if config.pull.show_list_post_pull {
    println!("Listing patch stack...");
    list::list(color).map_err(PullError::ListFailed)?
  }

  Ok(())
}
