use super::super::private::git;
use super::super::super::ps;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use super::super::private::state_management;

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
  GetPatchStatePathFailed(state_management::PatchStatesPathError),
  ReadPatchStatesFailed(state_management::ReadPatchStatesError),
  CommitMissing,
  GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchStatusFailed(PatchStatusError)
}

pub enum PatchStatus {
  WithoutBranch,
  BranchCreated,
  BranchCreatedButHasChanged,
  PushedToRemote,
  PushedToRemoteButHasChanged,
  RequestedReview,
  RequestedReviewButHasChanged,
  Integrated
}

pub fn list() -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(ListError::GetPatchStackFailed)?;
    let list_of_patches = ps::get_patch_list(&repo, patch_stack).map_err(ListError::GetPatchListFailed)?;

    let patch_meta_data_path = state_management::patch_states_path(&repo).map_err(ListError::GetPatchStatePathFailed)?;
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
  GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError)
}

fn patch_status(patch_meta_data_option: Option<&state_management::Patch>, repo: &git2::Repository, commit_diff_patch_id: git2::Oid) -> Result<PatchStatus, PatchStatusError> {
  match patch_meta_data_option {
    None => Ok(PatchStatus::WithoutBranch),
    Some(patch_meta_data) => {
      match &patch_meta_data.state {
        state_management::PatchState::BranchCreated(rr_branch_name) => {
          // get the singular commit (a.k.a) patch that should be the head of rr_branch_name
          let commit = git::singular_commit_of_branch(repo, rr_branch_name, git2::BranchType::Local).map_err(PatchStatusError::SingularCommitOfBrachFailure)?;
          // get it's diff_patch_id
          let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
          // compare it to the diff_patch_id of the current patch
          if remote_commit_diff_patch_id != commit_diff_patch_id {
            Ok(PatchStatus::BranchCreatedButHasChanged)
          } else {
            Ok(PatchStatus::BranchCreated)
          }
        },
        state_management::PatchState::PushedToRemote(rr_branch_name) => {
          // get the singular commit (a.k.a) patch that should be the head of rr_branch_name
          let commit = git::singular_commit_of_branch(repo, rr_branch_name, git2::BranchType::Remote).map_err(PatchStatusError::SingularCommitOfBrachFailure)?;
          // get it's diff_patch_id
          let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
          // compare it to the diff_patch_id of the current patch
          if remote_commit_diff_patch_id != commit_diff_patch_id {
            Ok(PatchStatus::PushedToRemoteButHasChanged)
          } else {
            Ok(PatchStatus::PushedToRemote)
          }
        },
        state_management::PatchState::RequestedReview(rr_branch_name) => {
          // get the singular commit (a.k.a) patch that should be the head of rr_branch_name
          let commit = git::singular_commit_of_branch(repo, rr_branch_name, git2::BranchType::Remote).map_err(PatchStatusError::SingularCommitOfBrachFailure)?;
          // get it's diff_patch_id
          let remote_commit_diff_patch_id = git::commit_diff_patch_id(repo, &commit).map_err(PatchStatusError::GetCommitDiffPatchIdFailed)?;
          // compare it to the diff_patch_id of the current patch
          if remote_commit_diff_patch_id != commit_diff_patch_id {
            Ok(PatchStatus::RequestedReviewButHasChanged)
          } else {
            Ok(PatchStatus::RequestedReview)
          }
        },
        state_management::PatchState::Published(_) => Ok(PatchStatus::Integrated)
      }
    }
  }
}

fn patch_status_to_string(patch_status: PatchStatus) -> String {
  match patch_status {
    PatchStatus::WithoutBranch => "   ",
    PatchStatus::BranchCreated => "b  ",
    PatchStatus::BranchCreatedButHasChanged => "b+ ",
    PatchStatus::PushedToRemote => "p  ",
    PatchStatus::PushedToRemoteButHasChanged => "p+ ",
    PatchStatus::RequestedReview => "rr ",
    PatchStatus::RequestedReviewButHasChanged => "rr+",
    PatchStatus::Integrated => "int"
  }.to_string()
}
