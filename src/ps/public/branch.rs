use super::super::super::ps;
use super::super::private::git;
use git2;
use std::result::Result;

#[derive(Debug)]
pub enum BranchError {
  OpenRepositoryFailed(git::CreateCwdRepositoryError),
  GetPatchStackFailed(ps::PatchStackError),
  PatchStackBaseNotFound,
  GetPatchListFailed(ps::GetPatchListError),
  PatchIndexNotFound,
  CreateBranchFailed(git2::Error),
  BranchNameNotUtf8,
  PatchSummaryMissing,
  FindCommitFailed(git2::Error),
  GetCommitParentZeroFailed(git2::Error),
  CherryPickFailed(git::GitError),
  OpenGitConfigFailed(git2::Error)
}

pub fn branch(
    start_patch_index: usize,
    end_patch_index_option: Option<usize>,
    branch_name: Option<String>
) -> Result<(), BranchError>  {
    let repo = git::create_cwd_repo().map_err(BranchError::OpenRepositoryFailed)?;
    let config = git2::Config::open_default().map_err(BranchError::OpenGitConfigFailed)?;

    // find the base of the current patch stack
    let patch_stack = ps::get_patch_stack(&repo).map_err(BranchError::GetPatchStackFailed)?;
    let patch_stack_base_commit = patch_stack
        .base
        .peel_to_commit()
        .map_err(|_| BranchError::PatchStackBaseNotFound)?;

  // find the patch series in the patch stack
  let patches_vec =
      ps::get_patch_list(&repo, &patch_stack).map_err(BranchError::GetPatchListFailed)?;
  let start_patch_oid = patches_vec
      .get(start_patch_index)
      .ok_or(BranchError::PatchIndexNotFound)?
      .oid;

  // translate the patch series to bounds for the cherry-pick range
  let start_patch_commit = repo
      .find_commit(start_patch_oid)
      .map_err(BranchError::FindCommitFailed)?;
  let start_patch_parent_commit = start_patch_commit
      .parent(0)
      .map_err(BranchError::GetCommitParentZeroFailed)?;
  let start_patch_parent_oid = start_patch_parent_commit
      .id();

  // Generate a branch name if one was not provided
  let branch_name = match branch_name {
    Some(name) => name,
    None => {
      let mut branch_name = String::from("gps-branch-");
      let patch_summary = start_patch_commit
          .summary().ok_or(BranchError::PatchSummaryMissing)?;
      let default_branch_name = ps::generate_rr_branch_name(patch_summary);
      branch_name.push_str(&default_branch_name);
      branch_name
    }
  };

  if let Some(end_patch_index) = end_patch_index_option {
    // find the patch series in the patch stack
    let end_patch_oid = patches_vec
        .get(end_patch_index)
        .ok_or(BranchError::PatchIndexNotFound)?
        .oid;

    // create a branch on the base of the current patch stack
    let branch = repo.branch(branch_name
        .as_str(), &patch_stack_base_commit, false)
        .map_err(BranchError::CreateBranchFailed)?;
    let branch_ref_name = branch
        .get()
        .name()
        .ok_or(BranchError::BranchNameNotUtf8)?;

    // cherry-pick the series of patches into the new branch
    git::cherry_pick_no_working_copy_range(&repo, &config, end_patch_oid, start_patch_parent_oid, branch_ref_name).map_err(BranchError::CherryPickFailed)?;
    git::cherry_pick_no_working_copy(&repo, &config, end_patch_oid, branch_ref_name, 0).map_err(BranchError::CherryPickFailed)?;
  } else {
    // create a branch on the base of the current patch stack
    let branch = repo.branch(branch_name
        .as_str(), &patch_stack_base_commit, false)
        .map_err(BranchError::CreateBranchFailed)?;
    let branch_ref_name = branch
        .get()
        .name()
        .ok_or(BranchError::BranchNameNotUtf8)?;

    // cherry-pick the single patch into the new branch
    git::cherry_pick_no_working_copy(&repo, &config, start_patch_oid, branch_ref_name, 0).map_err(BranchError::CherryPickFailed)?;
  }

    Ok(())
}
