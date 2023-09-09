use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::state_management;
use super::super::private::utils;
use super::verify_isolation;
use std::fmt;
use std::path::PathBuf;
use std::result::Result;

#[derive(Debug)]
pub enum RequestReviewError {
    OpenRepositoryFailed(git::CreateCwdRepositoryError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    BranchNameNotUtf8,
    PostSyncHookNotFound,
    PostSyncHookNotExecutable(PathBuf),
    FindHookFailed(hooks::FindHookError),
    SyncFailed(ps::public::sync::SyncError),
    FetchPatchMetaDataFailed(state_management::FetchPatchMetaDataError),
    PatchMetaDataMissing,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    HookExecutionFailed(utils::ExecuteError),
    StorePatchStateFailed(state_management::StorePatchStateError),
    GetConfigFailed(config::GetConfigError),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    FindPatchCommitFailed(ps::FindPatchCommitError),
    PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    FindRemoteRequestReviewBranchFailed(git2::Error),
    GetRemoteCommitFailed(git2::Error),
    RemoteCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    FindRemoteFailed(git2::Error),
    RemoteUrlNotUtf8,
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
            Self::FetchPatchMetaDataFailed(e) => {
                write!(f, "Failed to fetch patch meta data - {:?}", e)
            }
            Self::PatchMetaDataMissing => write!(f, "Patch meta data unexpectedly missing"),
            Self::CurrentBranchNameMissing => write!(f, "Current branch name unexpectedly missin"),
            Self::GetUpstreamBranchNameFailed => write!(f, "Failed to get upstream branch name"),
            Self::GetRemoteNameFailed => write!(f, "Failed te get remote name"),
            Self::HookExecutionFailed(e) => write!(
                f,
                "Execution of the request_review_post_sync hook failed - {:?}",
                e
            ),
            Self::StorePatchStateFailed(e) => {
                write!(f, "Failed to store updated patch state - {:?}", e)
            }
            Self::GetConfigFailed(e) => write!(f, "Failed to get Git Patch Stack config - {:?}", e),
            Self::IsolationVerificationFailed(e) => {
                write!(f, "Isolation verification failed - {:?}", e)
            }
            Self::FindPatchCommitFailed(e) => write!(f, "Failed to find patch commit - {:?}", e),
            Self::PatchCommitDiffPatchIdFailed(e) => {
                write!(f, "Failed to get diff patch identifier - {:?}", e)
            }
            Self::FindRemoteRequestReviewBranchFailed(e) => {
                write!(f, "Failed to find remote request review branch - {:?}", e)
            }
            Self::GetRemoteCommitFailed(e) => write!(f, "Failed to get remote commit - {:?}", e),
            Self::RemoteCommitDiffPatchIdFailed(e) => {
                write!(f, "Failed to get remote commit's diff patch id - {:?}", e)
            }
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
    // check if post_request_review hook exists
    let repo = git::create_cwd_repo().map_err(RequestReviewError::OpenRepositoryFailed)?;

    let patch_commit = ps::find_patch_commit(&repo, patch_index)
        .map_err(RequestReviewError::FindPatchCommitFailed)?;
    let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit)
        .map_err(RequestReviewError::PatchCommitDiffPatchIdFailed)?;

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
    let hook_path = hooks::find_hook(repo_root_str, repo_gitdir_str, "request_review_post_sync")?;

    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(RequestReviewError::GetConfigFailed)?;

    if config.request_review.verify_isolation {
        verify_isolation::verify_isolation(patch_index, None, color)
            .map_err(RequestReviewError::IsolationVerificationFailed)?;
    }

    // sync patch up to remote
    let (created_branch_name, ps_id) = ps::public::sync::sync(patch_index, given_branch_name)
        .map_err(RequestReviewError::SyncFailed)?;

    // get arguments for the hook
    let patch_meta_data = state_management::fetch_patch_meta_data(&repo, &ps_id)
        .map_err(RequestReviewError::FetchPatchMetaDataFailed)?
        .ok_or(RequestReviewError::PatchMetaDataMissing)?;

    let cur_branch_name =
        git::get_current_branch(&repo).ok_or(RequestReviewError::CurrentBranchNameMissing)?;
    let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str())
        .map_err(|_| RequestReviewError::GetUpstreamBranchNameFailed)?;
    let remote_name = repo
        .branch_remote_name(&branch_upstream_name)
        .map_err(|_| RequestReviewError::GetRemoteNameFailed)?;
    let remote_name_str = remote_name
        .as_str()
        .ok_or(RequestReviewError::BranchNameNotUtf8)?;
    let remote = repo
        .find_remote(remote_name_str)
        .map_err(RequestReviewError::FindRemoteFailed)?;
    let remote_url_str = remote.url().ok_or(RequestReviewError::RemoteUrlNotUtf8)?;

    let pattern = format!(
        "refs/remotes/{}/",
        remote_name
            .as_str()
            .ok_or(RequestReviewError::BranchNameNotUtf8)?
    );
    let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");

    let remote_rr_branch = repo
        .find_branch(
            format!("{}/{}", remote_name_str, created_branch_name).as_str(),
            git2::BranchType::Remote,
        )
        .map_err(RequestReviewError::FindRemoteRequestReviewBranchFailed)?;
    let remote_commit = remote_rr_branch
        .get()
        .peel_to_commit()
        .map_err(RequestReviewError::GetRemoteCommitFailed)?;
    let remote_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &remote_commit)
        .map_err(RequestReviewError::RemoteCommitDiffPatchIdFailed)?;

    // execute the hook
    utils::execute(
        hook_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?,
        &[
            rerequesting_review(&patch_meta_data),
            &created_branch_name,
            &upstream_branch_shorthand,
            &remote_name_str,
            &remote_url_str,
        ],
    )
    .map_err(RequestReviewError::HookExecutionFailed)?;

    // update patch state to indicate that we have requested review
    let mut new_patch_meta_data = patch_meta_data;
    new_patch_meta_data.state = state_management::PatchState::RequestedReview(
        remote_name
            .as_str()
            .ok_or(RequestReviewError::BranchNameNotUtf8)?
            .to_string(),
        created_branch_name,
        patch_commit_diff_patch_id.to_string(),
        remote_commit_diff_patch_id.to_string(),
    );
    state_management::store_patch_state(&repo, &new_patch_meta_data)
        .map_err(RequestReviewError::StorePatchStateFailed)?;

    Ok(())
}

fn rerequesting_review(patch_meta_data: &state_management::Patch) -> &'static str {
    if patch_meta_data.state.has_requested_review() {
        "true"
    } else {
        "false"
    }
}
