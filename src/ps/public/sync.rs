use super::super::private::git;
use super::super::super::ps;
use super::super::private::state_management;
use uuid::Uuid;

#[derive(Debug)]
pub enum SyncError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  CreateRrBranchFailed(ps::private::request_review_branch::RequestReviewBranchError),
  RequestReviewBranchNameMissing,
  ForcePushFailed(git::ExtForcePushError),
  StorePatchStateFailed(state_management::StorePatchStateError),
  FindPatchCommitFailed(ps::FindPatchCommitError),
  PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  FindRemoteRequestReviewBranchFailed(git2::Error),
  BranchNameNotUtf8,
  GetRemoteCommitFailed(git2::Error),
  RemoteCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
}

pub fn sync(patch_index: usize, given_branch_name: Option<String>) -> Result<(String, Uuid), SyncError> {
  let repo = git::create_cwd_repo().map_err(|_| SyncError::RepositoryNotFound)?;

  let patch_commit = ps::find_patch_commit(&repo, patch_index).map_err(SyncError::FindPatchCommitFailed)?;
  let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).map_err(SyncError::PatchCommitDiffPatchIdFailed)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(SyncError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| SyncError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| SyncError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  let (branch, ps_id) = ps::private::request_review_branch::request_review_branch(&repo, patch_index, given_branch_name).map_err(SyncError::CreateRrBranchFailed)?;

  let branch_ref_name = branch.get().shorthand().ok_or(SyncError::RequestReviewBranchNameMissing)?;
  let rr_branch_name = branch_ref_name.to_string();

  // force push request review branch up to remote
  git::ext_push(true, remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).map_err(SyncError::ForcePushFailed)?;

  let remote_name_str = remote_name.as_str().ok_or(SyncError::BranchNameNotUtf8)?;
  let remote_rr_branch = repo.find_branch(format!("{}/{}", remote_name_str, rr_branch_name).as_str(), git2::BranchType::Remote).map_err(SyncError::FindRemoteRequestReviewBranchFailed)?;
  let remote_commit = remote_rr_branch.get().peel_to_commit().map_err(SyncError::GetRemoteCommitFailed)?;
  let remote_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &remote_commit).map_err(SyncError::RemoteCommitDiffPatchIdFailed)?;

  // associate the patch to the branch that was created
  let rr_branch_name_copy = rr_branch_name.clone();
  state_management::update_patch_state(&repo, &ps_id, |patch_meta_data_option|
    match patch_meta_data_option {
      Some(patch_meta_data) => {
        match patch_meta_data.state {
          state_management::PatchState::Integrated(_, _, _) => patch_meta_data,
          state_management::PatchState::RequestedReview(_, _, _, _) => {
            state_management::Patch {
              patch_id: ps_id,
              state: state_management::PatchState::RequestedReview(remote_name.as_str().unwrap().to_string(), rr_branch_name_copy, patch_commit_diff_patch_id.to_string(), remote_commit_diff_patch_id.to_string())
            }
          },
          _ => {
            state_management::Patch {
              patch_id: ps_id,
              state: state_management::PatchState::PushedToRemote(remote_name.as_str().unwrap().to_string(), rr_branch_name_copy, patch_commit_diff_patch_id.to_string(), remote_commit_diff_patch_id.to_string())
            }
          }
        }
      },
      None => {
        state_management::Patch {
          patch_id: ps_id,
          state: state_management::PatchState::PushedToRemote(remote_name.as_str().unwrap().to_string(), rr_branch_name_copy, patch_commit_diff_patch_id.to_string(), remote_commit_diff_patch_id.to_string())
        }
      }
    }
  ).map_err(SyncError::StorePatchStateFailed)?;

  Ok((rr_branch_name, ps_id))
}
