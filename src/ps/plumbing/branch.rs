use crate::ps::state_management;

use super::git;
use super::super::super::ps;
use uuid::Uuid;
use std::result::Result;
use std::fmt;

#[derive(Debug)]
pub enum BranchError {
  RepositoryMissing,
  PatchStackNotFound,
  PatchStackBaseNotFound,
  PatchIndexNotFound,
  PatchCommitNotFound,
  PatchMessageMissing,
  AddPsIdToPatchFailed(ps::AddPsIdError),
  PatchSummaryMissing,
  CreateRrBranchFailed,
  RrBranchNameNotUtf8,
  CherryPickFailed(git::GitError),
  GetPatchListFailed(ps::GetPatchListError),
  GetPatchMetaDataPathFailed(state_management::PatchStatesPathError),
  ReadPatchMetaDataFailed(state_management::ReadPatchStatesError),
  WritePatchMetaDataFailed(state_management::WritePatchStatesError)
}

impl From<git::CreateCwdRepositoryError> for BranchError {
  fn from(_e: git::CreateCwdRepositoryError) -> Self {
    BranchError::RepositoryMissing
  }
}

impl From<ps::PatchStackError> for BranchError {
  fn from(e: ps::PatchStackError) -> Self {
    match e {
      ps::PatchStackError::GitError(_git2_error) => BranchError::PatchStackNotFound,
      ps::PatchStackError::HeadNoName => BranchError::PatchStackNotFound,
      ps::PatchStackError::UpstreamBranchNameNotFound => BranchError::PatchStackNotFound,
    }
  }
}

impl From<ps::AddPsIdError> for BranchError {
  fn from(e: ps::AddPsIdError) -> Self {
    BranchError::AddPsIdToPatchFailed(e)
  }
}

impl fmt::Display for BranchError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      BranchError::RepositoryMissing => write!(f, "Repository not found in current working directory"),
      BranchError::PatchStackNotFound => write!(f, "Patch Stack not found"),
      BranchError::PatchStackBaseNotFound => write!(f, "Patch Stack Base not found"),
      BranchError::PatchIndexNotFound => write!(f, "Patch Index out of range"),
      BranchError::PatchCommitNotFound => write!(f, "Patch commit not found"),
      BranchError::PatchMessageMissing => write!(f, "Patch missing message"),
      BranchError::AddPsIdToPatchFailed(_add_ps_id_error) => write!(f, "Failed to add patch stack id to patch"),
      BranchError::PatchSummaryMissing => write!(f, "Patch missing summary"),
      BranchError::CreateRrBranchFailed => write!(f, "Failed to create request-review branch"),
      BranchError::RrBranchNameNotUtf8 => write!(f, "request-review branch is not utf8"),
      BranchError::CherryPickFailed(_git_error) => write!(f, "Failed to cherry pick"),
      BranchError::GetPatchListFailed(_patch_list_error) => write!(f, "Failed to get patch list"),
      BranchError::GetPatchMetaDataPathFailed(_patch_meta_data_path_error) => write!(f, "Failed to get patch meta data path {:?}", _patch_meta_data_path_error),
      BranchError::ReadPatchMetaDataFailed(_read_patch_meta_data_error) => write!(f, "Failed to read patch meta data {:?}", _read_patch_meta_data_error),
      BranchError::WritePatchMetaDataFailed(_write_patch_meta_data_error) => write!(f, "Failed to write patch meta data {:?}", _write_patch_meta_data_error)
    }
  }
}


pub fn branch<'a>(repo: &'a git2::Repository, patch_index: usize) -> Result<(git2::Branch<'a>, Uuid), BranchError>  {
  // - find the patch identified by the patch_index
  let patch_stack = ps::get_patch_stack(&repo)?;
  let patch_stack_base_commit = patch_stack.base.peel_to_commit().map_err(|_| BranchError::PatchStackBaseNotFound)?;
  let patches_vec = ps::get_patch_list(&repo, patch_stack).map_err(|e| BranchError::GetPatchListFailed(e))?;
  let patch_oid = patches_vec.get(patch_index).ok_or(BranchError::PatchIndexNotFound)?.oid;
  let patch_commit = repo.find_commit(patch_oid).map_err(|_| BranchError::PatchCommitNotFound)?;

  let patch_message = patch_commit.message().ok_or(BranchError::PatchMessageMissing)?;

  // fetch or add patch id given patch_message
  let new_patch_oid: git2::Oid;
  let ps_id: Uuid;
  if let Some(extracted_ps_id) = ps::extract_ps_id(patch_message) {
    ps_id = extracted_ps_id;
    new_patch_oid = patch_oid;
  } else {
    ps_id = Uuid::new_v4();
    new_patch_oid = ps::add_ps_id(&repo, patch_oid, ps_id)?;
  }

  // fetch patch meta data given repo and patch_id
  let patch_meta_data_path = state_management::patch_states_path(repo).map_err(|e| BranchError::GetPatchMetaDataPathFailed(e))?;
  let mut patch_meta_data = state_management::read_patch_states(&patch_meta_data_path).map_err(|e| BranchError::ReadPatchMetaDataFailed(e))?;
  let branch_name = match patch_meta_data.get(&ps_id) {
    Some(patch_meta_data) => patch_meta_data.state.branch_name(),
    None => {
      println!("generating branch name");
      let patch_summary = patch_commit.summary().ok_or(BranchError::PatchSummaryMissing)?;
      ps::generate_rr_branch_name(patch_summary)
    }
  };

  let branch = repo.branch(branch_name.as_str(), &patch_stack_base_commit, true).map_err(|_| BranchError::CreateRrBranchFailed)?;
  
  let branch_ref_name = branch.get().name().ok_or(BranchError::RrBranchNameNotUtf8)?;

  // - cherry pick the patch onto new rr branch
  git::cherry_pick_no_working_copy(&repo, new_patch_oid, branch_ref_name).map_err(BranchError::CherryPickFailed)?;

  // record new patch state
  let new_patch_meta_data = state_management::Patch {
    patch_id: ps_id,
    state: state_management::PatchState::BranchCreated(branch_name)
  };
  patch_meta_data.insert(ps_id, new_patch_meta_data);
  state_management::write_patch_states(&patch_meta_data_path, &patch_meta_data).map_err(|e| BranchError::WritePatchMetaDataFailed(e))?;

  Ok((branch, ps_id))
}
