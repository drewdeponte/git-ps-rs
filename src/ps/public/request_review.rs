use super::super::super::ps;
use super::super::private::hooks;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::utils;
use super::super::private::state_management;
use std::result::Result;

#[derive(Debug)]
pub enum RequestReviewError {
  OpenRepositoryFailed(git::CreateCwdRepositoryError),
  GetRepoRootPathFailed(paths::PathsError),
  PathNotUtf8,
  BranchNameNotUtf8,
  HookNotFound(hooks::FindHookError),
  SyncFailed(ps::public::sync::SyncError),
  FetchPatchMetaDataFailed(state_management::FetchPatchMetaDataError),
  PatchMetaDataMissing,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  HookExecutionFailed(utils::ExecuteError),
  StorePatchStateFailed(state_management::StorePatchStateError)
}

pub fn request_review(patch_index: usize, given_branch_name: Option<String>) -> Result<(), RequestReviewError> {
  // check if post_request_review hook exists
  let repo = git::create_cwd_repo().map_err(RequestReviewError::OpenRepositoryFailed)?;

  // find post_request_review hook
  let repo_root_path = paths::repo_root_path(&repo).map_err(RequestReviewError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?;
  let hook_path = hooks::find_hook(repo_root_str, "request_review_post_sync").map_err(RequestReviewError::HookNotFound)?;

  // sync patch up to remote
  let (created_branch_name, ps_id) = ps::public::sync::sync(patch_index, given_branch_name).map_err(RequestReviewError::SyncFailed)?;

  // get arguments for the hook
  let patch_meta_data = state_management::fetch_patch_meta_data(&repo, &ps_id).map_err(RequestReviewError::FetchPatchMetaDataFailed)?.ok_or(RequestReviewError::PatchMetaDataMissing)?;

  let cur_branch_name = git::get_current_branch(&repo).ok_or(RequestReviewError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| RequestReviewError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| RequestReviewError::GetRemoteBranchNameFailed)?;

  let pattern = format!("refs/remotes/{}/", remote_name.as_str().ok_or(RequestReviewError::BranchNameNotUtf8)?);
  let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");

  // execute the hook
  utils::execute(hook_path.to_str().ok_or(RequestReviewError::PathNotUtf8)?, &[rerequesting_review(&patch_meta_data), &created_branch_name, &upstream_branch_shorthand]).map_err(RequestReviewError::HookExecutionFailed)?;

  // update patch state to indicate that we have requested review
  let mut new_patch_meta_data = patch_meta_data;
  new_patch_meta_data.state = state_management::PatchState::RequestedReview(remote_name.as_str().ok_or(RequestReviewError::BranchNameNotUtf8)?.to_string(), created_branch_name);
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
