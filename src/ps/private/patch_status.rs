use super::commit_is_behind::{commit_is_behind, CommitIsBehindError};
use super::git;
use super::state_management;

pub enum PatchStatus {
  WithoutBranch,
  BranchCreated,
  BranchCreatedButLocalHasChanged,
  PushedToRemote,
  PushedToRemoteButLocalHasChanged,
  PushedToRemoteButRemoteHasChanged,
  PushedToRemoteButBothHaveChanged,
  PushedToRemoteNowBehind,
  PushedToRemoteNowBehindButLocalHasChanged,
  RequestedReview,
  RequestedReviewButLocalHasChanged,
  RequestedReviewButRemoteHasChanged,
  RequestedReviewButBothHaveChanged,
  RequestedReviewNowBehind,
  RequestedReviewNowBehindButLocalHasChanged,
  Integrated
}

#[derive(Debug)]
pub enum PatchStatusError {
  SingularCommitOfBrachFailure(git::SingularCommitOfBranchError),
  GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchIdFromPatchIdStringFailed(git2::Error),
  GetCommitIsBehindFailed(CommitIsBehindError)
}

fn compute_branched_status(local_patch_has_changed: bool) -> PatchStatus {
  if local_patch_has_changed {
    PatchStatus::BranchCreatedButLocalHasChanged
  } else {
    PatchStatus::BranchCreated
  }
}

fn compute_pushed_to_remote_status(local_patch_has_changed: bool, remote_patch_has_changed: bool, patch_is_behind: Option<bool>) -> PatchStatus {
  let is_behind = patch_is_behind.unwrap_or(false);
  if local_patch_has_changed && remote_patch_has_changed {
    PatchStatus::PushedToRemoteButBothHaveChanged
  } else if local_patch_has_changed && is_behind {
    PatchStatus::PushedToRemoteNowBehindButLocalHasChanged
  } else if local_patch_has_changed {
    PatchStatus::PushedToRemoteButLocalHasChanged
  } else if remote_patch_has_changed {
    PatchStatus::PushedToRemoteButRemoteHasChanged
  } else if is_behind {
    PatchStatus::PushedToRemoteNowBehind
  } else {
    PatchStatus::PushedToRemote
  }
}

fn compute_request_reviewed_status(local_patch_has_changed: bool, remote_patch_has_changed: bool, patch_is_behind: Option<bool>) -> PatchStatus {
  let is_behind = patch_is_behind.unwrap_or(false);
  if local_patch_has_changed && remote_patch_has_changed {
    PatchStatus::RequestedReviewButBothHaveChanged
  } else if local_patch_has_changed && is_behind {
    PatchStatus::RequestedReviewNowBehindButLocalHasChanged
  } else if local_patch_has_changed {
    PatchStatus::RequestedReviewButLocalHasChanged
  } else if remote_patch_has_changed {
    PatchStatus::RequestedReviewButRemoteHasChanged
  } else if is_behind {
    PatchStatus::RequestedReviewNowBehind
  } else {
    PatchStatus::RequestedReview
  }
}

pub fn patch_status(patch_meta_data_option: Option<&state_management::Patch>, repo: &git2::Repository, commit_diff_patch_id: git2::Oid, patch_stack_base_oid: git2::Oid) -> Result<PatchStatus, PatchStatusError> {
  match patch_meta_data_option {
    None => Ok(PatchStatus::WithoutBranch),
    Some(patch_meta_data) => {
      match &patch_meta_data.state {
        state_management::PatchState::BranchCreated(rr_branch_name, operation_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, rr_branch_name, git2::BranchType::Local, patch_stack_base_oid) {
            Ok(_) => {
              Ok(compute_branched_status(local_patch_has_changed))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              Ok(compute_branched_status(local_patch_has_changed))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::PushedToRemote(remote, rr_branch_name, operation_diff_patch_id_string, operation_remote_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let operation_remote_diff_patch_id = git2::Oid::from_str(operation_remote_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, format!("{}/{}", remote, rr_branch_name).as_str(), git2::BranchType::Remote, patch_stack_base_oid) {
            Ok(commit) => {
              let is_behind = commit_is_behind(&commit, patch_stack_base_oid).map_err(PatchStatusError::GetCommitIsBehindFailed)?;

              let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
              let remote_patch_has_changed = remote_commit_diff_patch_id != operation_remote_diff_patch_id;

              Ok(compute_pushed_to_remote_status(local_patch_has_changed, remote_patch_has_changed, Option::Some(is_behind)))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              let remote_patch_has_changed = true;
              Ok(compute_pushed_to_remote_status(local_patch_has_changed, remote_patch_has_changed, Option::None))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::RequestedReview(remote, rr_branch_name, operation_diff_patch_id_string, operation_remote_diff_patch_id_string) => {
          let operation_diff_patch_id = git2::Oid::from_str(operation_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let operation_remote_diff_patch_id = git2::Oid::from_str(operation_remote_diff_patch_id_string).map_err(PatchStatusError::PatchIdFromPatchIdStringFailed)?;
          let local_patch_has_changed = commit_diff_patch_id != operation_diff_patch_id;

          match git::singular_commit_of_branch(repo, format!("{}/{}", remote, rr_branch_name).as_str(), git2::BranchType::Remote, patch_stack_base_oid) {
            Ok(commit) => {
              let is_behind = commit_is_behind(&commit, patch_stack_base_oid).map_err(PatchStatusError::GetCommitIsBehindFailed)?;

              let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
              let remote_patch_has_changed = remote_commit_diff_patch_id != operation_remote_diff_patch_id;
              Ok(compute_request_reviewed_status(local_patch_has_changed, remote_patch_has_changed, Option::Some(is_behind)))
            },
            Err(git::SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(_, _)) => {
              let remote_patch_has_changed = true;
              Ok(compute_request_reviewed_status(local_patch_has_changed, remote_patch_has_changed, Option::None))
            },
            Err(e) => Err(PatchStatusError::SingularCommitOfBrachFailure(e))
          }
        },
        state_management::PatchState::Integrated(_, _, _) => Ok(PatchStatus::Integrated)
      }
    }
  }
}
