use super::super::private::config;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::utils;
use super::super::public::fetch;
use super::super::public::list;

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
    ListFailed(list::ListError),
}

impl std::fmt::Display for PullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepositoryMissing => write!(f, "repository missing"),
            Self::GetHeadBranchNameFailed => write!(f, "get head branch name failed"),
            Self::GetUpstreamBranchNameFailed => write!(f, "get upstream branch name failed"),
            Self::RebaseFailed(e) => write!(f, "rebase failed, {}", e),
            Self::FetchFailed(e) => write!(f, "fetch failed, {}", e),
            Self::GetRepoRootPathFailed(e) => write!(f, "get repository root path failed, {}", e),
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::GetConfigFailed(e) => write!(f, "get config failed, {}", e),
            Self::ListFailed(e) => write!(f, "get list failed, {}", e),
        }
    }
}

impl std::error::Error for PullError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RepositoryMissing => None,
            Self::GetHeadBranchNameFailed => None,
            Self::GetUpstreamBranchNameFailed => None,
            Self::RebaseFailed(e) => Some(e),
            Self::FetchFailed(e) => Some(e),
            Self::GetRepoRootPathFailed(e) => Some(e),
            Self::PathNotUtf8 => None,
            Self::GetConfigFailed(e) => Some(e),
            Self::ListFailed(e) => Some(e),
        }
    }
}

pub fn pull(color: bool) -> Result<(), PullError> {
    let repo = git::create_cwd_repo().map_err(|_| PullError::RepositoryMissing)?;

    let repo_root_path = paths::repo_root_path(&repo).map_err(PullError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(PullError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(PullError::PathNotUtf8)?;
    let config =
        config::get_config(repo_root_str, repo_gitdir_str).map_err(PullError::GetConfigFailed)?;

    let head_ref = repo
        .head()
        .map_err(|_| PullError::GetHeadBranchNameFailed)?;
    let head_branch_shorthand = head_ref
        .shorthand()
        .ok_or(PullError::GetHeadBranchNameFailed)?;
    let head_branch_name = head_ref.name().ok_or(PullError::GetHeadBranchNameFailed)?;

    let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name)
        .map_err(|_| PullError::GetUpstreamBranchNameFailed)?;

    println!("Fetching upstream patches...");
    fetch::fetch(color).map_err(PullError::FetchFailed)?;
    println!();

    println!("Rebasing...");
    utils::execute(
        "git",
        &[
            "rebase",
            "--no-reapply-cherry-picks",
            "--onto",
            upstream_branch_name.as_str(),
            upstream_branch_name.as_str(),
            head_branch_shorthand,
        ],
    )
    .map_err(PullError::RebaseFailed)?;
    println!();

    if config.pull.show_list_post_pull {
        println!("Listing patch stack...");
        list::list(color).map_err(PullError::ListFailed)?
    }

    Ok(())
}
