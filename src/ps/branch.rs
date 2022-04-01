
use super::utils;
use super::git;
use super::ps;
use uuid::Uuid;
use std::result::Result;

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
  CherryPickFailed(git::GitError)
}

impl From<git::CreateCwdRepositoryError> for BranchError {
  fn from(e: git::CreateCwdRepositoryError) -> Self {
    BranchError::RepositoryMissing
  }
}

impl From<ps::PatchStackError> for BranchError {
  fn from(e: ps::PatchStackError) -> Self {
    match e {
      ps::PatchStackError::GitError(git2_error) => BranchError::PatchStackNotFound,
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

pub fn branch(patch_index: usize) -> Result<(), BranchError>  {
  let repo = git::create_cwd_repo()?;

  // - find the patch identified by the patch_index
  let patch_stack = ps::get_patch_stack(&repo)?;
  let patch_stack_base_commit = patch_stack.base.peel_to_commit().map_err(|_| BranchError::PatchStackBaseNotFound)?;
  let patches_vec = ps::get_patch_list(&repo, patch_stack);

  let patch_oid = patches_vec.get(patch_index).ok_or(BranchError::PatchIndexNotFound)?.oid;
  println!("patch_oid: {}", patch_oid);

  let patch_commit = repo.find_commit(patch_oid).map_err(|_| BranchError::PatchCommitNotFound)?;
  let patch_message = patch_commit.message().ok_or(BranchError::PatchMessageMissing)?;

  // - if patch doesn't have patch id, add one
  let new_patch_oid: git2::Oid;
  if let Some(ps_id) = ps::extract_ps_id(patch_message) {
    new_patch_oid = patch_oid;
  } else {
    // add patch stack id to the commit
    new_patch_oid = ps::add_ps_id(&repo, patch_oid, Uuid::new_v4())?;
  }

  // - create rr branch based on upstream branch
  let patch_summary = patch_commit.summary().ok_or(BranchError::PatchSummaryMissing)?;
  let branch_name = ps::generate_rr_branch_name(patch_summary);
  let branch = repo.branch(branch_name.as_str(), &patch_stack_base_commit, false).map_err(|_| BranchError::CreateRrBranchFailed)?;
  
  let branch_ref_name = branch.get().name().ok_or(BranchError::RrBranchNameNotUtf8)?;
  println!("branch_ref_name: {}", branch_ref_name);

  // TODO: finish handling errors below this

  // - cherry pick the patch onto new rr branch
  let cherry_picked_patch_oid = git::cherry_pick_no_working_copy(&repo, new_patch_oid, branch_ref_name).map_err(BranchError::CherryPickFailed)?;
  println!("cherry_picked_patch_oid: {}", cherry_picked_patch_oid);

  Ok(())
}
