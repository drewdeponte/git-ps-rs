use super::super::private::git;
use super::super::super::ps;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use super::super::private::state_management;
use super::super::private::paths;
use ansi_term::Colour::{Green, Yellow, Cyan};
use super::super::private::patch_status;

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
  PatchStatusFailed(patch_status::PatchStatusError),
  GetPatchStackBaseTargetFailed
}

pub fn list(color: bool) -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(ListError::GetPatchStackFailed)?;
    let list_of_patches = ps::get_patch_list(&repo, &patch_stack).map_err(ListError::GetPatchListFailed)?;

    let patch_meta_data_path = paths::patch_states_path(&repo).map_err(ListError::GetPatchStatePathFailed)?;
    let patch_meta_data = state_management::read_patch_states(patch_meta_data_path).map_err(ListError::ReadPatchStatesFailed)?;

    let patch_stack_base_oid = patch_stack.base.target().ok_or(ListError::GetPatchStackBaseTargetFailed)?;

    for patch in list_of_patches.into_iter().rev() {
        let commit = repo.find_commit(patch.oid).map_err(|_| ListError::CommitMissing)?;
        let patch_state = match ps::commit_ps_id(&commit) {
          Some(ps_id) => patch_meta_data.get(&ps_id),
          None => None 
        };

        let commit_diff_patch_id = git::commit_diff_patch_id(&repo, &commit).map_err(ListError::GetCommitDiffPatchIdFailed)?;
        let patch_status = patch_status::patch_status(patch_state, &repo, commit_diff_patch_id, patch_stack_base_oid).map_err(ListError::PatchStatusFailed)?;
        let patch_status_string = patch_status_to_string(patch_status);

        let patch_str = format!("{:<4}", patch.index);
        let patch_status_str = format!("{:<6}", patch_status_string);
        let patch_oid_str = format!("{:.6}", patch.oid);

        if color {
          println!("{:<4} {:<6} {:.6} {}", Green.paint(patch_str), Cyan.paint(patch_status_str), Yellow.paint(patch_oid_str), patch.summary);
        } else {
          println!("{:<4} {:<6} {:.6} {}", patch_str, patch_status_str, patch_oid_str, patch.summary);
        }
    }

    Ok(())
}

fn patch_status_to_string(patch_status: patch_status::PatchStatus) -> String {
  match patch_status {
    patch_status::PatchStatus::WithoutBranch                                  => "     ",
    patch_status::PatchStatus::BranchCreated                                  => "b    ",
    patch_status::PatchStatus::BranchCreatedButLocalHasChanged                => "b+   ",
    patch_status::PatchStatus::PushedToRemote                                 => "s    ",
    patch_status::PatchStatus::PushedToRemoteButLocalHasChanged               => "s+   ",
    patch_status::PatchStatus::PushedToRemoteButRemoteHasChanged              => "s  ! ",
    patch_status::PatchStatus::PushedToRemoteButBothHaveChanged               => "s+ ! ",
    patch_status::PatchStatus::PushedToRemoteNowBehind                        => "s   ↓",
    patch_status::PatchStatus::PushedToRemoteNowBehindButLocalHasChanged      => "s+  ↓",
    patch_status::PatchStatus::RequestedReview                                => "rr   ",
    patch_status::PatchStatus::RequestedReviewButLocalHasChanged              => "rr+  ",
    patch_status::PatchStatus::RequestedReviewButRemoteHasChanged             => "rr ! ",
    patch_status::PatchStatus::RequestedReviewButBothHaveChanged              => "rr+! ",
    patch_status::PatchStatus::RequestedReviewNowBehind                       => "rr  ↓",
    patch_status::PatchStatus::RequestedReviewNowBehindButLocalHasChanged     => "rr+ ↓",
    patch_status::PatchStatus::Integrated                                     => "int  "
  }.to_string()
}
