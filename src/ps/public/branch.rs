use super::super::super::ps;
use super::super::private::git;
use git2;
use std::result::Result;
use super::super::private::utils;
use super::super::private::paths;
use super::super::private::hooks;
use super::super::private::string_file_io::{write_str_to_file, WriteStrToFileError, ReadStringFromFileError};

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
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteNameFailed,
  GetRemoteBranchNameFailed,
  GetCurrentBranchFailed,
  PathNotUtf8,
  FailedToCheckout(utils::ExecuteError),
  GetIsolateLastBranchPathFailed(paths::PathsError),
  StoreLastBranchFailed(WriteStrToFileError),
  ReadLastBranchFailed(ReadStringFromFileError),
  GetRepoRootPathFailed(paths::PathsError),
  HookExecutionFailed(utils::ExecuteError),
  HookNotFound(hooks::FindHookError),
  ForcePushFailed(git::ExtForcePushError),
  FindCommitFailed(git2::Error),
  GetCommitParentZeroFailed(git2::Error),
  CherryPickFailed(git::GitError),
  OpenGitConfigFailed(git2::Error)
}

pub fn branch(
    color:bool,
    start_patch_index: usize,
    end_patch_index_option: Option<usize>,
    branch_name: Option<String>,
    create_remote: bool
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

  // Create upstream name for branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(BranchError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| BranchError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| BranchError::GetRemoteBranchNameFailed)?;

  // create a branch on the base of the current patch stack
  let branch = repo
      .branch(branch_name.as_str(), &patch_stack_base_commit, true)
      .map_err(BranchError::CreateBranchFailed)?;
  let branch_ref_name = branch
      .get()
      .name()
      .ok_or(BranchError::BranchNameNotUtf8)?;

  if let Some(end_patch_index) = end_patch_index_option {
    // find the patch series in the patch stack
    let end_patch_oid = patches_vec
        .get(end_patch_index)
        .ok_or(BranchError::PatchIndexNotFound)?
        .oid;

    // cherry-pick the series of patches into the new branch
    git::cherry_pick_no_working_copy_range(&repo, &config, end_patch_oid, start_patch_parent_oid, branch_ref_name).map_err(BranchError::CherryPickFailed)?;
    git::cherry_pick_no_working_copy(&repo, &config, end_patch_oid, branch_ref_name, 0).map_err(BranchError::CherryPickFailed)?;
  } else {
    // cherry-pick the single patch into the new branch
    git::cherry_pick_no_working_copy(&repo, &config, start_patch_oid, branch_ref_name, 0).map_err(BranchError::CherryPickFailed)?;
  }

  // get currently checked out branch name
  let checked_out_branch = git::get_current_branch_shorthand(&repo).ok_or(BranchError::GetCurrentBranchFailed)?;
  // write currently checked out branch name to disk
  let path = paths::isolate_last_branch_path(&repo).map_err(BranchError::GetIsolateLastBranchPathFailed)?;
  write_str_to_file(checked_out_branch.as_str(), path).map_err(BranchError::StoreLastBranchFailed)?;

  // checkout the branch
  utils::execute("git", &["checkout", &branch_name]).map_err(BranchError::FailedToCheckout)?;

  let repo_root_path = paths::repo_root_path(&repo).map_err(BranchError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(BranchError::PathNotUtf8)?;
  match hooks::find_hook(repo_root_str, "isolate_post_checkout") {
    Ok(hook_path) => utils::execute(hook_path.to_str().ok_or(BranchError::PathNotUtf8)?, &[]).map_err(BranchError::HookExecutionFailed)?,
    Err(hooks::FindHookError::NotFound) => {
      utils::print_warn(color,
r#"
The isolate_post_checkout hook was not found!

This hook is NOT required but it is strongly recommended that you set it
up. It is executed after the temporary isolation branch has been created,
the patch cherry-picked into it and the isolation branch checked out.

It is intended to be used to further verify patch isolation by verifying
that your code bases build succeeds and your test suite passes.

You can effectively have it do whatever you want as it is just a hook.
An exit status of 0, success, informs gps that the further isolation
verification was successful. Any non-zero exit status will indicate failure
and cause gps to abort.

You can find more information and examples of this hook and others at
the following.

https://book.git-ps.sh/tool/hooks.html
"#)
    },
    Err(hooks::FindHookError::NotExecutable(hook_path)) => {
      let path_str = hook_path.to_str().unwrap_or("unknow path");
      let msg = format!(
r#"
The isolate_post_checkout hook was found at

{}

but it is NOT executable. Due to this the hook is being skipped. Generally
this can be corrected with the following.

chmod u+x {}
"#, path_str, path_str);
      utils::print_warn(color, &msg);
    },
    Err(e) => return Err(BranchError::HookNotFound(e))
  }

  // checkout the branch
  utils::execute("git", &["checkout", &checked_out_branch]).map_err(BranchError::FailedToCheckout)?;

  // If the user wants to create a remote branch, create it by force pushing
  if create_remote {
    git::ext_push(true, remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).map_err(BranchError::ForcePushFailed)?;
  }

  Ok(())
}
