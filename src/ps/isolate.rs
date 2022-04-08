use std::result::Result;
use super::git;
use super::super::ps;
use super::plumbing::utils;

#[derive(Debug)]
pub enum IsolateError {
  OpenGitRepositoryFailed(git::CreateCwdRepositoryError),
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

pub fn isolate(patch_index_optional: Option<usize>) -> Result<(), IsolateError> {
  let isolate_branch_name = "ps/tmp/isolate";
  let repo = ps::plumbing::git::create_cwd_repo().map_err(IsolateError::OpenGitRepositoryFailed)?;
  match patch_index_optional {
    Some(patch_index) => {
      let patch_stack = ps::get_patch_stack(&repo).map_err(IsolateError::GetPatchStackFailed)?;
      let patch_stack_base_commit = patch_stack.base.peel_to_commit().map_err(|_| IsolateError::PatchStackBaseNotFound)?;
      let patches_vec = ps::get_patch_list(&repo, patch_stack).map_err(IsolateError::GetPatchListFailed)?;
      let patch_oid = patches_vec.get(patch_index).ok_or(IsolateError::PatchIndexNotFound)?.oid;

      let branch = repo.branch(isolate_branch_name, &patch_stack_base_commit, true).map_err(|_| IsolateError::CreateBranchFailed)?;

      let branch_ref_name = branch.get().name().ok_or(IsolateError::BranchNameNotUtf8)?;

      // - cherry pick the patch onto new rr branch
      git::cherry_pick_no_working_copy(&repo, patch_oid, branch_ref_name).map_err(|_| IsolateError::CherryPickFailed)?;

      // TODO:
      // store state of the branch currently checked out on so that when the
      // command is next run and not provided a patch index it will simply read
      // the state of the previously checked out branch and check that branch out.

      // checkout the ps/tmp/checkout branch
      utils::execute("git", &["checkout", isolate_branch_name]).map_err(IsolateError::FailedToCheckout)?;
      Ok(())
    },
    None => {
      // TODO: attempt to retreive state of the previous branch
      // if succeeed then check that branch out
      // if fail to retreive previous state then error out
      Err(IsolateError::PatchIndexNotProvided)
    }
  }
}
