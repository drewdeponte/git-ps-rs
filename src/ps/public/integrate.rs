use super::super::private::git;
use super::super::super::ps;
use super::super::private::state_management;

#[derive(Debug)]
pub enum IntegrateError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  CreateRrBranchFailed(ps::private::branch::BranchError),
  RequestReviewBranchNameMissing,
  ForcePushFailed(ps::private::git::ExtForcePushError),
  PushFailed(ps::private::git::ExtForcePushError),
  GetShortBranchNameFailed,
  ConvertStringToStrFailed,
  UpdatePatchMetaDataFailed(state_management::StorePatchStateError)
}

pub fn integrate(patch_index: usize) -> Result<(), IntegrateError> {
  let repo = git::create_cwd_repo().map_err(|_| IntegrateError::RepositoryNotFound)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| IntegrateError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  let (branch, ps_id) = ps::private::branch::branch(&repo, patch_index).map_err(|e| IntegrateError::CreateRrBranchFailed(e))?;

  // force push request review branch up to remote
  let branch_ref_name = branch.get().name().ok_or(IntegrateError::RequestReviewBranchNameMissing)?;
  let short_branch_name = branch.get().shorthand().ok_or(IntegrateError::GetShortBranchNameFailed)?.to_string();
  git::ext_push(true, remote_name.as_str().ok_or(IntegrateError::ConvertStringToStrFailed)?, branch_ref_name, branch_ref_name).map_err(|e| IntegrateError::ForcePushFailed(e))?;

  // - push rr branch up to upstream branch (e.g. origin/main)
  let pattern = format!("refs/remotes/{}/", remote_name.as_str().ok_or(IntegrateError::ConvertStringToStrFailed)?);
  let remote_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");
  git::ext_push(false, remote_name.as_str().ok_or(IntegrateError::ConvertStringToStrFailed)?, branch_ref_name, &remote_branch_shorthand).map_err(|e| IntegrateError::PushFailed(e))?;

  // associate the patch to the branch that was created
  state_management::update_patch_state(&repo, &ps_id, |patch_meta_data_option|
    match patch_meta_data_option {
      Some(patch_meta_data) => {
        match patch_meta_data.state {
          state_management::PatchState::Published(ref _branch_name) => patch_meta_data.clone(),
          _ => {
            state_management::Patch {
              patch_id: ps_id,
              state: state_management::PatchState::Published(short_branch_name)
            }
          }
        }
      },
      None => {
        state_management::Patch {
          patch_id: ps_id,
          state: state_management::PatchState::Published(short_branch_name)
        }
      }
    }
  ).map_err(|e| IntegrateError::UpdatePatchMetaDataFailed(e))?;

  Ok(())
}
