use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::utils;
use super::verify_isolation;
use std::fmt;
use std::path::PathBuf;
use std::result::Result;

#[derive(Debug)]
pub enum RequestReviewError {
    OpenRepositoryFailed(git::CreateCwdRepositoryError),
    FindPatchCommitFailed(ps::FindPatchCommitError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    SyncFailed(ps::public::sync::SyncError),
    CurrentPatchStackBranchNameMissing,
    GetCurrentPatchStackUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    BranchNameNotUtf8,
    FindRemoteFailed(git2::Error),
    RemoteUrlNotUtf8,
    HookExecutionFailed(utils::ExecuteError),
    PostSyncHookNotFound,
    PostSyncHookNotExecutable(PathBuf),
    FindHookFailed(hooks::FindHookError),
}

impl From<hooks::FindHookError> for RequestReviewError {
    fn from(e: hooks::FindHookError) -> Self {
        match e {
            hooks::FindHookError::NotFound => Self::PostSyncHookNotFound,
            hooks::FindHookError::NotExecutable(path) => Self::PostSyncHookNotExecutable(path),
            hooks::FindHookError::PathExpandHomeFailed(_) => Self::FindHookFailed(e),
            hooks::FindHookError::HomeDirNotFound => Self::FindHookFailed(e),
        }
    }
}

impl fmt::Display for RequestReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenRepositoryFailed(e) => write!(
                f,
                "Repository not found in current working directory - {:?}",
                e
            ),
            Self::GetRepoRootPathFailed(e) => write!(f, "Get repository path failed - {:?}", e),
            Self::PathNotUtf8 => write!(
                f,
                "Failed to process repository root path as it is NOT utf8"
            ),
            Self::BranchNameNotUtf8 => write!(f, "Failed to process remote name as it is NOT utf8"),
            Self::PostSyncHookNotFound => write!(f, "request_review_post_sync hook not found"),
            Self::PostSyncHookNotExecutable(path) => write!(
                f,
                "request_review_post_sync hook - {} - is not executable",
                path.to_str().unwrap_or("unknown path")
            ),
            Self::FindHookFailed(e) => {
                write!(f, "failed to find request_review_post_sync hook - {:?}", e)
            }
            Self::SyncFailed(e) => write!(f, "Failed to sync patch to remote - {:?}", e),
            Self::CurrentPatchStackBranchNameMissing => {
                write!(f, "Current branch name unexpectedly missin")
            }
            Self::GetCurrentPatchStackUpstreamBranchNameFailed => {
                write!(f, "Failed to get upstream branch name")
            }
            Self::GetRemoteNameFailed => write!(f, "Failed te get remote name"),
            Self::HookExecutionFailed(e) => write!(
                f,
                "Execution of the request_review_post_sync hook failed - {:?}",
                e
            ),
            Self::GetConfigFailed(e) => write!(f, "Failed to get Git Patch Stack config - {:?}", e),
            Self::IsolationVerificationFailed(e) => {
                write!(f, "Isolation verification failed - {:?}", e)
            }
            Self::FindPatchCommitFailed(e) => write!(f, "Failed to find patch commit - {:?}", e),
            Self::RemoteUrlNotUtf8 => write!(f, "Failed to process remote url as it is NOT utf8"),
            Self::FindRemoteFailed(e) => {
                write!(f, "Failed to find remote - {:?}", e)
            }
        }
    }
}

pub fn request_review(
    patch_index: usize,
    given_branch_name: Option<String>,
    color: bool,
) -> Result<(), RequestReviewError> {
    let repo = git::create_cwd_repo().map_err(RequestReviewError::OpenRepositoryFailed)?;

    // find post_request_review hook
    let repo_root_path =
        paths::repo_root_path(&repo).map_err(RequestReviewError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path
        .to_str()
        .ok_or(RequestReviewError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path
        .to_str()
        .ok_or(RequestReviewError::PathNotUtf8)?;
    let request_review_post_sync_hook_path =
        hooks::find_hook(repo_root_str, repo_gitdir_str, "request_review_post_sync")?;

    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(RequestReviewError::GetConfigFailed)?;

    // verify isolation
    if config.request_review.verify_isolation {
        verify_isolation::verify_isolation(patch_index, None, color)
            .map_err(RequestReviewError::IsolationVerificationFailed)?;
    }

    // sync patch up to remote
    let (patch_upstream_branch_name, _patch_upstream_branch_remote_name, _ps_id) =
        ps::public::sync::sync(patch_index, given_branch_name)
            .map_err(RequestReviewError::SyncFailed)?;

    // execute post sync hook
    let cur_patch_stack_branch_name = git::get_current_branch(&repo)
        .ok_or(RequestReviewError::CurrentPatchStackBranchNameMissing)?;
    let cur_patch_stack_upstream_branch_name =
        git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
            .map_err(|_| RequestReviewError::GetCurrentPatchStackUpstreamBranchNameFailed)?;
    let cur_patch_stack_upstream_branch_remote_name = repo
        .branch_remote_name(&cur_patch_stack_upstream_branch_name)
        .map_err(|_| RequestReviewError::GetRemoteNameFailed)?;
    let cur_patch_stack_upstream_branch_remote_name_str =
        cur_patch_stack_upstream_branch_remote_name
            .as_str()
            .ok_or(RequestReviewError::BranchNameNotUtf8)?;
    let cur_patch_stack_upstream_branch_remote = repo
        .find_remote(cur_patch_stack_upstream_branch_remote_name_str)
        .map_err(RequestReviewError::FindRemoteFailed)?;
    let cur_patch_stack_upstream_branch_remote_url_str = cur_patch_stack_upstream_branch_remote
        .url()
        .ok_or(RequestReviewError::RemoteUrlNotUtf8)?;

    let pattern = format!(
        "refs/remotes/{}/",
        cur_patch_stack_upstream_branch_remote_name_str
    );
    let cur_patch_stack_upstream_branch_name_relative_to_remote =
        str::replace(&cur_patch_stack_upstream_branch_name, pattern.as_str(), "");

    utils::execute(
        request_review_post_sync_hook_path
            .to_str()
            .ok_or(RequestReviewError::PathNotUtf8)?,
        &[
            &patch_upstream_branch_name,
            &cur_patch_stack_upstream_branch_name_relative_to_remote,
            cur_patch_stack_upstream_branch_remote_name_str,
            cur_patch_stack_upstream_branch_remote_url_str,
        ],
    )
    .map_err(RequestReviewError::HookExecutionFailed)?;

    Ok(())
}
