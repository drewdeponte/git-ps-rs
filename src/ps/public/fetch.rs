use super::super::private::config;
use super::super::private::git;
use super::super::private::paths;
use super::super::public::upstream_patches;

#[derive(Debug)]
pub enum FetchError {
    FetchFailed(git::ExtFetchError),
    UpstreamPatchesFailure(upstream_patches::UpstreamPatchesError),
    RepositoryMissing,
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FetchFailed(e) => write!(f, "fetch failed, {}", e),
            Self::UpstreamPatchesFailure(e) => write!(f, "get upstream patches failed, {}", e),
            Self::RepositoryMissing => write!(f, "repository missing"),
            Self::GetRepoRootPathFailed(e) => write!(f, "get repository root path failed, {}", e),
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::GetConfigFailed(e) => write!(f, "get config failed, {}", e),
        }
    }
}

impl std::error::Error for FetchError {}

pub fn fetch(color: bool) -> Result<(), FetchError> {
    git::ext_fetch().map_err(FetchError::FetchFailed)?;

    let repo = git::create_cwd_repo().map_err(|_| FetchError::RepositoryMissing)?;

    let repo_root_path = paths::repo_root_path(&repo).map_err(FetchError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(FetchError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(FetchError::PathNotUtf8)?;
    let config =
        config::get_config(repo_root_str, repo_gitdir_str).map_err(FetchError::GetConfigFailed)?;

    if config.fetch.show_upstream_patches_after_fetch {
        upstream_patches::upstream_patches(color).map_err(FetchError::UpstreamPatchesFailure)?;
    }

    Ok(())
}
