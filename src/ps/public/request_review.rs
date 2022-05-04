use super::super::super::ps;
use super::super::private::hooks;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::utils;
use super::super::private::state_management;
use super::super::private::config;
use super::super::private::verify_isolation;
use std::result::Result;
use std::fmt;
use std::path::PathBuf;

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
  PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError)
}

impl From<hooks::FindHookError> for RequestReviewError {
  fn from(e: hooks::FindHookError) -> Self {
    match e {
      hooks::FindHookError::NotFound => Self::PostSyncHookNotFound,
      hooks::FindHookError::NotExecutable(path) => Self::PostSyncHookNotExecutable(path),
      hooks::FindHookError::PathExpandHomeFailed(_) => Self::FindHookFailed(e)
    }
  }
}

impl fmt::Display for RequestReviewError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::OpenRepositoryFailed(e) => write!(f, "Repository not found in current working directory - {:?}", e),
      Self::GetRepoRootPathFailed(e) => write!(f, "Get repository path failed - {:?}", e),
      Self::PathNotUtf8 => write!(f, "Failed to process repository root path as it is NOT utf8"),
      Self::BranchNameNotUtf8 => write!(f, "Failed to process remote name as it is NOT utf8"),
      Self::PostSyncHookNotFound => write!(f, "request_review_post_sync hook not found"),
      Self::PostSyncHookNotExecutable(path) => write!(f, "request_review_post_sync hook - {} - is not executable", path.to_str().unwrap_or("unknown path")),
      Self::FindHookFailed(e) => write!(f, "failed to find request_review_post_sync hook - {:?}", e),
      Self::SyncFailed(e) => write!(f, "Failed to sync patch to remote - {:?}", e),
      Self::FetchPatchMetaDataFailed(e) => write!(f, "Failed to fetch patch meta data - {:?}", e),
      Self::PatchMetaDataMissing => write!(f, "Patch meta data unexpectedly missing"),
      Self::CurrentBranchNameMissing => write!(f, "Current branch name unexpectedly missin"),
      Self::GetUpstreamBranchNameFailed => write!(f, "Failed to get upstream branch name"),
      Self::GetRemoteNameFailed => write!(f, "Failed te get remote name"),
      Self::HookExecutionFailed(e) => write!(f, "Execution of the request_review_post_sync hook failed - {:?}", e),
      Self::StorePatchStateFailed(e) => write!(f, "Failed to store updated patch state - {:?}", e),
      Self::GetConfigFailed(e) => write!(f, "Failed to get Git Patch Stack config - {:?}", e),
      Self::IsolationVerificationFailed(e) => write!(f, "Isolation verification failed - {:?}", e),
      Self::FindPatchCommitFailed(e) => write!(f, "Failed to find patch commit - {:?}", e),
      Self::PatchCommitDiffPatchIdFailed(e) => write!(f, "Failed to get diff patch identifier - {:?}", e)
    }
  }
}

pub fn request_review(patch_index: usize, given_branch_name: Option<String>) -> Result<(), RequestReviewError> {
  // check if post_request_review hook exists
  let repo = git::create_cwd_repo().map_err(RequestReviewError::OpenRepositoryFailed)?;

  let patch_commit = ps::find_patch_commit(&repo, patch_index).map_err(RequestReviewError::FindPatchCommitFailed)?;
  let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).map_err(RequestReviewError::PatchCommitDiffPatchIdFailed)?;

  // find post_request_review hook
  let repo_root_path = paths::repo_root_path(&repo).map_err(RequestReviewError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?;
  let hook_path = hooks::find_hook(repo_root_str, "request_review_post_sync")?;

  let config = config::get_config(repo_root_str).map_err(RequestReviewError::GetConfigFailed)?;

  if config.request_review.verify_isolation {
    verify_isolation::verify_isolation(patch_index).map_err(RequestReviewError::IsolationVerificationFailed)?;
  }

  // sync patch up to remote
  let (created_branch_name, ps_id) = ps::public::sync::sync(patch_index, given_branch_name).map_err(RequestReviewError::SyncFailed)?;

  // get arguments for the hook
  let patch_meta_data = state_management::fetch_patch_meta_data(&repo, &ps_id).map_err(RequestReviewError::FetchPatchMetaDataFailed)?.ok_or(RequestReviewError::PatchMetaDataMissing)?;

  let cur_branch_name = git::get_current_branch(&repo).ok_or(RequestReviewError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| RequestReviewError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| RequestReviewError::GetRemoteNameFailed)?;

  let pattern = format!("refs/remotes/{}/", remote_name.as_str().ok_or(RequestReviewError::BranchNameNotUtf8)?);
  let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");

  // execute the hook
  utils::execute(hook_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?, &[rerequesting_review(&patch_meta_data), &created_branch_name, &upstream_branch_shorthand]).map_err(RequestReviewError::HookExecutionFailed)?;

  // update patch state to indicate that we have requested review
  let mut new_patch_meta_data = patch_meta_data;
  new_patch_meta_data.state = state_management::PatchState::RequestedReview(remote_name.as_str().ok_or(RequestReviewError::BranchNameNotUtf8)?.to_string(), created_branch_name, patch_commit_diff_patch_id.to_string());
  state_management::store_patch_state(&repo, &new_patch_meta_data).map_err(RequestReviewError::StorePatchStateFailed)?;

  Ok(())
}

fn rerequesting_review(patch_meta_data: &state_management::Patch) -> &'static str {
  if patch_meta_data.state.has_requested_review() {
    "true"
  } else {
    "false"
  }
}
