use std::result::Result;
use super::git;
use super::super::ps;
use super::plumbing::utils;

#[derive(Debug)]
pub enum CheckoutError {
  GetPatchStackFailed(ps::PatchStackError),
  GetPatchListFailed(ps::GetPatchListError),
  PatchIndexNotFound,
  PatchStackBaseNotFound,
  CreateBranchFailed,
  BranchNameNotUtf8,
  CherryPickFailed,
  FailedToCheckout(utils::ExecuteError),
  PatchIndexNotProvided
}

pub fn checkout(repo: &git2::Repository, patch_index_optional: Option<usize>) -> Result<(), CheckoutError> {
  match patch_index_optional {
    Some(patch_index) => {
      let patch_stack = ps::get_patch_stack(repo).map_err(CheckoutError::GetPatchStackFailed)?;
      let patch_stack_base_commit = patch_stack.base.peel_to_commit().map_err(|_| CheckoutError::PatchStackBaseNotFound)?;
      let patches_vec = ps::get_patch_list(repo, patch_stack).map_err(CheckoutError::GetPatchListFailed)?;
      let patch_oid = patches_vec.get(patch_index).ok_or(CheckoutError::PatchIndexNotFound)?.oid;

      let branch = repo.branch("ps/tmp/checkout", &patch_stack_base_commit, true).map_err(|_| CheckoutError::CreateBranchFailed)?;

      let branch_ref_name = branch.get().name().ok_or(CheckoutError::BranchNameNotUtf8)?;

      // - cherry pick the patch onto new rr branch
      git::cherry_pick_no_working_copy(repo, patch_oid, branch_ref_name).map_err(|_| CheckoutError::CherryPickFailed)?;

      // TODO:
      // store state of the branch currently checked out on so that when the
      // command is next run and not provided a patch index it will simply read
      // the state of the previously checked out branch and check that branch out.
      
      // checkout the ps/tmp/checkout branch
      utils::execute("git", &["checkout", "ps/tmp/checkout"]).map_err(CheckoutError::FailedToCheckout)?;
      Ok(())
    },
    None => {
      // TODO: attempt to retreive state of the previous branch
      // if succeeed then check that branch out
      // if fail to retreive previous state then error out
      Err(CheckoutError::PatchIndexNotProvided)
    }
  }
}
