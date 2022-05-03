use super::super::private::git;
use super::super::super::ps;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use super::super::private::state_management;
use super::super::private::paths;

#[derive(Serialize, Deserialize, Debug)]
struct RequestReviewRecord {
    patch_stack_id: Uuid,
    branch_name: String,
    commit_id: String,
    published: Option<bool>,
    location_agnostic_hash: Option<String>
}

#[derive(Debug)]
pub enum ListError {
  RepositoryNotFound,
  GetPatchStackFailed(ps::PatchStackError),
  GetPatchListFailed(ps::GetPatchListError),
  GetPatchStatePathFailed(paths::PathsError),
  ReadPatchStatesFailed(state_management::ReadPatchStatesError),
  CommitMissing,
  GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchStatusFailed(PatchStatusError)
}

pub enum PatchStatus {
  WithoutBranch,
  BranchCreated,
  BranchCreatedButLocalHasChanged,
  BranchCreatedButRemoteHasChanged,
  BranchCreatedButBothHaveChanged,
  PushedToRemote,
  PushedToRemoteButLocalHasChanged,
  PushedToRemoteButRemoteHasChanged,
  PushedToRemoteButBothHaveChanged,
  RequestedReview,
  RequestedReviewButLocalHasChanged,
  RequestedReviewButRemoteHasChanged,
  RequestedReviewButBothHaveChanged,
  Integrated
}

pub fn list() -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(ListError::GetPatchStackFailed)?;
    let list_of_patches = ps::get_patch_list(&repo, patch_stack).map_err(ListError::GetPatchListFailed)?;

    let patch_meta_data_path = paths::patch_states_path(&repo).map_err(ListError::GetPatchStatePathFailed)?;
    let patch_meta_data = state_management::read_patch_states(patch_meta_data_path).map_err(ListError::ReadPatchStatesFailed)?;

    for patch in list_of_patches.into_iter().rev() {
        let commit = repo.find_commit(patch.oid).map_err(|_| ListError::CommitMissing)?;
        let patch_state = match ps::commit_ps_id(&commit) {
          Some(ps_id) => patch_meta_data.get(&ps_id),
          None => None 
        };

        let commit_diff_patch_id = git::commit_diff_patch_id(&repo, &commit).map_err(ListError::GetCommitDiffPatchIdFailed)?;
        let patch_status = patch_status(patch_state, &repo, commit_diff_patch_id).map_err(ListError::PatchStatusFailed)?;
        let patch_status_string = patch_status_to_string(patch_status);

        println!("{:<4} {:<6} {:.6} {}", patch.index, patch_status_string, patch.oid, patch.summary);
    }

    Ok(())
}

#[derive(Debug)]
pub enum PatchStatusError {
  SingularCommitOfBrachFailure(git::SingularCommitOfBranchError),
  GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchIdFromPatchIdStringFailed(git2::Error)
}

fn compute_branched_status(local_patch_has_changed: bool, remote_patch_has_changed: bool) -> PatchStatus {
  if local_patch_has_changed && remote_patch_has_changed {
    PatchStatus::BranchCreatedButBothHaveChanged
  } else if local_patch_has_changed {
    PatchStatus::BranchCreatedButLocalHasChanged
  } else if remote_patch_has_changed {
    PatchStatus::BranchCreatedButRemoteHasChanged
  } else {
    PatchStatus::BranchCreated
  }
}

fn compute_pushed_to_remote_status(local_patch_has_changed: bool, remote_patch_has_changed: bool) -> PatchStatus {
  if local_patch_has_changed && remote_patch_has_changed {
    PatchStatus::PushedToRemoteButBothHaveChanged
  } else if local_patch_has_changed {
    PatchStatus::PushedToRemoteButLocalHasChanged
  } else if remote_patch_has_changed {
    PatchStatus::PushedToRemoteButRemoteHasChanged
  } else {
    PatchStatus::PushedToRemote
  }
}

fn compute_request_reviewed_status(local_patch_has_changed: bool, remote_patch_has_changed: bool) -> PatchStatus {
  if local_patch_has_changed && remote_patch_has_changed {
    PatchStatus::RequestedReviewButBothHaveChanged
  } else if local_patch_has_changed {
    PatchStatus::RequestedReviewButLocalHasChanged
  } else if remote_patch_has_changed {
    PatchStatus::RequestedReviewButRemoteHasChanged
  } else {
    PatchStatus::RequestedReview
  }
}

fn patch_status(patch_meta_data_option: Option<&state_management::Patch>, repo: &git2::Repository, commit_diff_patch_id: git2::Oid) -> Result<PatchStatus, PatchStatusError> {
  match patch_meta_data_option {
    None => Ok(PatchStatus::WithoutBranch),
    Some(patch_meta_data) => {
      match &patch_meta_data.state {
        state_management::PatchState::BranchCreated(rr_branch_name, operation_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, rr_branch_name, git2::BranchType::Local) {
            Ok(commit) => {
              let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
              let remote_patch_has_changed = remote_commit_diff_patch_id != operation_diff_patch_id; 
              Ok(compute_branched_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              let remote_patch_has_changed = true;
              Ok(compute_branched_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::PushedToRemote(remote, rr_branch_name, operation_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, format!("{}/{}", remote, rr_branch_name).as_str(), git2::BranchType::Remote) {
            Ok(commit) => {
              let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
              let remote_patch_has_changed = remote_commit_diff_patch_id != operation_diff_patch_id; 
              Ok(compute_pushed_to_remote_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              let remote_patch_has_changed = true;
              Ok(compute_pushed_to_remote_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::RequestedReview(remote, rr_branch_name, operation_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, format!("{}/{}", remote, rr_branch_name).as_str(), git2::BranchType::Remote) {
            Ok(commit) => {
              let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
              let remote_patch_has_changed = remote_commit_diff_patch_id != operation_diff_patch_id; 
              Ok(compute_request_reviewed_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              let remote_patch_has_changed = true;
              Ok(compute_request_reviewed_status(local_patch_has_changed, remote_patch_has_changed))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::Integrated(_, _, _) => Ok(PatchStatus::Integrated)
      }
    }
  }
}

fn patch_status_to_string(patch_status: PatchStatus) -> String {
  match patch_status {
    PatchStatus::WithoutBranch                      => "    ",
    PatchStatus::BranchCreated                      => "b   ",
    PatchStatus::BranchCreatedButLocalHasChanged    => "b+  ",
    PatchStatus::BranchCreatedButRemoteHasChanged   => "b  !",
    PatchStatus::BranchCreatedButBothHaveChanged    => "b+ !",
    PatchStatus::PushedToRemote                     => "s   ",
    PatchStatus::PushedToRemoteButLocalHasChanged   => "s+  ",
    PatchStatus::PushedToRemoteButRemoteHasChanged  => "s  !",
    PatchStatus::PushedToRemoteButBothHaveChanged   => "s+ !",
    PatchStatus::RequestedReview                    => "rr  ",
    PatchStatus::RequestedReviewButLocalHasChanged  => "rr+ ",
    PatchStatus::RequestedReviewButRemoteHasChanged => "rr !",
    PatchStatus::RequestedReviewButBothHaveChanged  => "rr+!",
    PatchStatus::Integrated                         => "int "
  }.to_string()
}
