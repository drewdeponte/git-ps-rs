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
