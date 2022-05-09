use std::result::Result;
use super::super::super::ps;
use super::super::private::utils;

#[derive(Debug)]
pub enum CheckoutError {
  GetPatchStackFailed(ps::PatchStackError),
  GetPatchListFailed(ps::GetPatchListError),
  PatchIndexNotFound,
  FailedToCheckout(utils::ExecuteError)
}

pub fn checkout(repo: &git2::Repository, patch_index: usize) -> Result<(), CheckoutError> {
  // get the sha of the commit referenced by the patch index
  let patch_stack = ps::get_patch_stack(repo).map_err(CheckoutError::GetPatchStackFailed)?;
  let patches_vec = ps::get_patch_list(repo, &patch_stack).map_err(CheckoutError::GetPatchListFailed)?;
  let patch_oid = patches_vec.get(patch_index).ok_or(CheckoutError::PatchIndexNotFound)?.oid;

  utils::execute("git", &["checkout", format!("{}", patch_oid).as_str()]).map_err(CheckoutError::FailedToCheckout)?;

  Ok(())
}
