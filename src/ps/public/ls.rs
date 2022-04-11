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
pub enum LsError {
  RepositoryNotFound,
  GetPatchStackFailed(ps::PatchStackError),
  GetPatchListFailed(ps::GetPatchListError),
  GetPatchStatePathFailed(state_management::PatchStatesPathError),
  ReadPatchStatesFailed(state_management::ReadPatchStatesError),
  CommitMissing
}

pub fn ls() -> Result<(), LsError> {
    let repo = git::create_cwd_repo().map_err(|_| LsError::RepositoryNotFound)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(|e| LsError::GetPatchStackFailed(e))?;
    let list_of_patches = ps::get_patch_list(&repo, patch_stack).map_err(|e| LsError::GetPatchListFailed(e))?;

    let patch_meta_data_path = state_management::patch_states_path(&repo).map_err(|e| LsError::GetPatchStatePathFailed(e))?;
    let patch_meta_data = state_management::read_patch_states(patch_meta_data_path).map_err(|e| LsError::ReadPatchStatesFailed(e))?;

    for patch in list_of_patches.into_iter().rev() {
        let patch_message = repo.find_commit(patch.oid).map_err(|_| LsError::CommitMissing)?.message().unwrap_or("").to_string();
        let patch_status = ps::extract_ps_id(&patch_message)
          .map_or("   ".to_string(), |patch_id| patch_status(patch_meta_data.get(&patch_id)));

        println!("{:<4} {:<6} {:.6} {}", patch.index, patch_status, patch.oid, patch.summary);
    }

    Ok(())
}

fn patch_status(patch_meta_data_option: Option<&state_management::Patch>) -> String {
  match patch_meta_data_option {
    None => "   ".to_string(),
    Some(patch_meta_data) => {
      match patch_meta_data.state {
        state_management::PatchState::BranchCreated(_) => "b  ".to_string(),
        state_management::PatchState::PushedToRemote(_) => "p  ".to_string(),
        state_management::PatchState::RequestedReview(_) => "rr ".to_string(),
        state_management::PatchState::Published(_) => "int".to_string()
      }
    }
  }
}
