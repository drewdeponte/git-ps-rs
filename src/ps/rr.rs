use super::git;
use super::super::ps;
use super::state_management::{Patch, PatchState, StorePatchStateError, store_patch_state};

#[derive(Debug)]
pub enum RequestReviewError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  CreateRrBranchFailed(ps::plumbing::branch::BranchError),
  RequestReviewBranchNameMissing,
  ForcePushFailed(ps::plumbing::git::ExtForcePushError),
  StorePatchStateFailed(StorePatchStateError)
}

pub fn rr(patch_index: usize) -> Result<(), RequestReviewError> {
  let repo = git::create_cwd_repo().map_err(|_| RequestReviewError::RepositoryNotFound)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(RequestReviewError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| RequestReviewError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| RequestReviewError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  let (branch, ps_id) = ps::plumbing::branch::branch(&repo, patch_index).map_err(|e| RequestReviewError::CreateRrBranchFailed(e))?;

  let branch_ref_name = branch.get().name().ok_or(RequestReviewError::RequestReviewBranchNameMissing)?;
  let rr_branch_name = branch_ref_name.to_string();

  // associate the patch to the branch that was created
  let patch_state = Patch {
    patch_id: ps_id,
    state: PatchState::PushedToRemote(rr_branch_name)
  };
  store_patch_state(&repo, &patch_state).map_err(|e| RequestReviewError::StorePatchStateFailed(e))?;

  // force push request review branch up to remote
  git::ext_push(true, remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).map_err(|e| RequestReviewError::ForcePushFailed(e))
}
