use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::utils;
use super::sync;
use super::verify_isolation;
use std::fmt;
use std::path::PathBuf;
use std::result::Result;

#[derive(Debug)]
pub enum RequestReviewError {
    OpenRepositoryFailed(Box<dyn std::error::Error>),
    GetRepoRootPathFailed(Box<dyn std::error::Error>),
    PathNotUtf8,
    GetConfigFailed(Box<dyn std::error::Error>),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    MergeCommitDetected(String),
    ConflictsExist(String, String),
    CurrentPatchStackBranchNameMissing,
    GetCurrentPatchStackUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    BranchNameNotUtf8,
    FindRemoteFailed(Box<dyn std::error::Error>),
    RemoteUrlNotUtf8,
    HookExecutionFailed(Box<dyn std::error::Error>),
    PostSyncHookNotExecutable(PathBuf),
    FindHookFailed(Box<dyn std::error::Error>),
    Unhandled(Box<dyn std::error::Error>),
}

impl From<sync::SyncError> for RequestReviewError {
    fn from(value: sync::SyncError) -> Self {
        match value {
            sync::SyncError::MergeCommitDetected(oid) => Self::MergeCommitDetected(oid),
            sync::SyncError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            _ => Self::Unhandled(value.into()),
        }
    }
}

impl From<verify_isolation::VerifyIsolationError> for RequestReviewError {
    fn from(value: verify_isolation::VerifyIsolationError) -> Self {
        match value {
            verify_isolation::VerifyIsolationError::MergeCommitDetected(oid) => {
                Self::MergeCommitDetected(oid)
            }
            verify_isolation::VerifyIsolationError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            _ => Self::IsolationVerificationFailed(value),
        }
    }
}

impl fmt::Display for RequestReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenRepositoryFailed(e) => write!(
                f,
                "Repository not found in current working directory - {}",
                e
            ),
            Self::GetRepoRootPathFailed(e) => write!(f, "Get repository path failed - {}", e),
            Self::PathNotUtf8 => write!(
                f,
                "Failed to process repository root path as it is NOT utf8"
            ),
            Self::BranchNameNotUtf8 => write!(f, "Failed to process remote name as it is NOT utf8"),
            Self::PostSyncHookNotExecutable(path) => write!(
                f,
                "request_review_post_sync hook - {} - is not executable",
                path.to_str().unwrap_or("unknown path")
            ),
            Self::FindHookFailed(e) => {
                write!(f, "failed to find request_review_post_sync hook - {}", e)
            }
            Self::CurrentPatchStackBranchNameMissing => {
                write!(f, "Current branch name unexpectedly missin")
            }
            Self::GetCurrentPatchStackUpstreamBranchNameFailed => {
                write!(f, "Failed to get upstream branch name")
            }
            Self::GetRemoteNameFailed => write!(f, "Failed te get remote name"),
            Self::HookExecutionFailed(e) => write!(
                f,
                "Execution of the request_review_post_sync hook failed - {}",
                e
            ),
            Self::GetConfigFailed(e) => write!(f, "Failed to get Git Patch Stack config - {}", e),
            Self::IsolationVerificationFailed(e) => {
                write!(f, "Isolation verification failed - {}", e)
            }

            Self::MergeCommitDetected(oid) => write!(f, "merge commit detected with sha {}", oid),
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict detected when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::RemoteUrlNotUtf8 => write!(f, "Failed to process remote url as it is NOT utf8"),
            Self::FindRemoteFailed(e) => {
                write!(f, "Failed to find remote - {}", e)
            }
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RequestReviewError {}

pub fn request_review(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    given_branch_name: Option<String>,
    color: bool,
    isolation_verification_hook: bool,
    post_sync_hook: bool,
) -> Result<(), RequestReviewError> {
    let repo =
        git::create_cwd_repo().map_err(|e| RequestReviewError::OpenRepositoryFailed(e.into()))?;

    // find post_request_review hook
    let repo_root_path = paths::repo_root_path(&repo)
        .map_err(|e| RequestReviewError::GetRepoRootPathFailed(e.into()))?;
    let repo_root_str = repo_root_path
        .to_str()
        .ok_or(RequestReviewError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path
        .to_str()
        .ok_or(RequestReviewError::PathNotUtf8)?;

    let mut post_sync_hook_path: Option<PathBuf> = None;
    if post_sync_hook {
        post_sync_hook_path =
            match hooks::find_hook(repo_root_str, repo_gitdir_str, "request_review_post_sync") {
                Ok(hook_path) => Some(hook_path),
                Err(hooks::FindHookError::NotFound) => None,
                Err(hooks::FindHookError::NotExecutable(p)) => {
                    return Err(RequestReviewError::PostSyncHookNotExecutable(p));
                }
                Err(e) => {
                    return Err(RequestReviewError::FindHookFailed(e.into()));
                }
            }
    }

    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(|e| RequestReviewError::GetConfigFailed(e.into()))?;

    // verify isolation
    if isolation_verification_hook && config.request_review.verify_isolation {
        verify_isolation::verify_isolation(start_patch_index, end_patch_index, color)?;
    }

    // sync patch up to remote
    let (patch_upstream_branch_name, _patch_upstream_branch_remote_name) =
        ps::public::sync::sync(start_patch_index, end_patch_index, given_branch_name)?;

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
        .map_err(|e| RequestReviewError::FindRemoteFailed(e.into()))?;
    let cur_patch_stack_upstream_branch_remote_url_str = cur_patch_stack_upstream_branch_remote
        .url()
        .ok_or(RequestReviewError::RemoteUrlNotUtf8)?;

    let pattern = format!(
        "refs/remotes/{}/",
        cur_patch_stack_upstream_branch_remote_name_str
    );
    let cur_patch_stack_upstream_branch_name_relative_to_remote =
        str::replace(&cur_patch_stack_upstream_branch_name, pattern.as_str(), "");

    if let Some(hook_path) = post_sync_hook_path {
        utils::execute(
            hook_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?,
            &[
                &patch_upstream_branch_name,
                &cur_patch_stack_upstream_branch_name_relative_to_remote,
                cur_patch_stack_upstream_branch_remote_name_str,
                cur_patch_stack_upstream_branch_remote_url_str,
            ],
        )
        .map_err(|e| RequestReviewError::HookExecutionFailed(e.into()))?;
    }

    Ok(())
}
