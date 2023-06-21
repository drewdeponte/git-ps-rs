use super::super::super::ps;
use super::super::private::cherry_picking;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::string_file_io::{
    read_str_from_file, write_str_to_file, ReadStringFromFileError, WriteStrToFileError,
};
use super::super::private::utils;
use std::result::Result;

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
    GetCurrentBranchFailed,
    StoreLastBranchFailed(WriteStrToFileError),
    ReadLastBranchFailed(ReadStringFromFileError),
    OpenGitConfigFailed(git2::Error),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    HookNotFound(hooks::FindHookError),
    HookExecutionFailed(utils::ExecuteError),
    UncommittedChangesExistFailure(git::UncommittedChangesError),
    UncommittedChangesExist,
    FindIsolateBranchFailed(git2::Error),
    DeleteIsolateBranchFailed(git2::Error),
    FailedToMapIndexesForCherryPick(cherry_picking::MapRangeForCherryPickError),
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    RemoteNameNotUtf8,
    FindRemoteFailed(git2::Error),
    RemoteUrlNotUtf8,
}

pub fn isolate(
    start_patch_index_optional: Option<usize>,
    end_patch_index_optional: Option<usize>,
    color: bool,
) -> Result<(), IsolateError> {
    let isolate_branch_name = "ps/tmp/isolate";
    let repo =
        ps::private::git::create_cwd_repo().map_err(IsolateError::OpenGitRepositoryFailed)?;

    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(IsolateError::PathNotUtf8)?;
    let config = git2::Config::open_default().map_err(IsolateError::OpenGitConfigFailed)?;

    if git::uncommitted_changes_exist(&repo)
        .map_err(IsolateError::UncommittedChangesExistFailure)?
    {
        return Err(IsolateError::UncommittedChangesExist);
    }

    match start_patch_index_optional {
        Some(patch_index) => {
            let patch_stack =
                ps::get_patch_stack(&repo).map_err(IsolateError::GetPatchStackFailed)?;
            let patches_vec = ps::get_patch_list(&repo, &patch_stack)
                .map_err(IsolateError::GetPatchListFailed)?;
            let patch_stack_base_commit = patch_stack
                .base
                .peel_to_commit()
                .map_err(|_| IsolateError::PatchStackBaseNotFound)?;

            let branch = repo
                .branch(isolate_branch_name, &patch_stack_base_commit, true)
                .map_err(|_| IsolateError::CreateBranchFailed)?;

            let branch_ref_name = branch.get().name().ok_or(IsolateError::BranchNameNotUtf8)?;

            // cherry pick the patch or patch range onto new isolation branch
            let cherry_pick_range = cherry_picking::map_range_for_cherry_pick(
                &patches_vec,
                patch_index,
                end_patch_index_optional,
            )
            .map_err(IsolateError::FailedToMapIndexesForCherryPick)?;

            git::cherry_pick(
                &repo,
                &config,
                cherry_pick_range.root_oid,
                cherry_pick_range.leaf_oid,
                branch_ref_name,
            )
            .map_err(|_| IsolateError::CherryPickFailed)?;

            // get currently checked out branch name
            let checked_out_branch = git::get_current_branch_shorthand(&repo)
                .ok_or(IsolateError::GetCurrentBranchFailed)?;
            // write currently checked out branch name to disk
            let path = paths::isolate_last_branch_path(&repo);
            write_str_to_file(checked_out_branch.as_str(), path)
                .map_err(IsolateError::StoreLastBranchFailed)?;

            let cur_branch_name =
                git::get_current_branch(&repo).ok_or(IsolateError::CurrentBranchNameMissing)?;
            let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str())
                .map_err(|_| IsolateError::GetUpstreamBranchNameFailed)?;
            let remote_name = repo
                .branch_remote_name(&branch_upstream_name)
                .map_err(|_| IsolateError::GetRemoteNameFailed)?;
            let remote_name_str = remote_name
                .as_str()
                .ok_or(IsolateError::RemoteNameNotUtf8)?;
            let remote = repo
                .find_remote(remote_name_str)
                .map_err(IsolateError::FindRemoteFailed)?;
            let remote_url_str = remote.url().ok_or(IsolateError::RemoteUrlNotUtf8)?;

            // checkout the ps/tmp/checkout branch
            utils::execute("git", &["checkout", isolate_branch_name])
                .map_err(IsolateError::FailedToCheckout)?;

            let repo_root_path =
                paths::repo_root_path(&repo).map_err(IsolateError::GetRepoRootPathFailed)?;
            let repo_root_str = repo_root_path.to_str().ok_or(IsolateError::PathNotUtf8)?;
            match hooks::find_hook(repo_root_str, repo_gitdir_str, "isolate_post_checkout") {
                Ok(hook_path) => utils::execute(
                    hook_path.to_str().ok_or(IsolateError::PathNotUtf8)?,
                    &[remote_name_str, remote_url_str],
                )
                .map_err(IsolateError::HookExecutionFailed)?,
                Err(hooks::FindHookError::NotFound) => utils::print_warn(
                    color,
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
"#,
                ),
                Err(hooks::FindHookError::NotExecutable(hook_path)) => {
                    let path_str = hook_path.to_str().unwrap_or("unknow path");
                    let msg = format!(
                        r#"
  The isolate_post_checkout hook was found at

    {}

  but it is NOT executable. Due to this the hook is being skipped. Generally
  this can be corrected with the following.

    chmod u+x {}
"#,
                        path_str, path_str
                    );
                    utils::print_warn(color, &msg);
                }
                Err(e) => return Err(IsolateError::HookNotFound(e)),
            }

            Ok(())
        }
        None => {
            // read last checked out branch name from disk
            let path = paths::isolate_last_branch_path(&repo);
            let last_branch =
                read_str_from_file(path).map_err(IsolateError::ReadLastBranchFailed)?;

            // check it out
            utils::execute("git", &["checkout", &last_branch])
                .map_err(IsolateError::FailedToCheckout)?;

            let mut isolate_branch = repo
                .find_branch(isolate_branch_name, git2::BranchType::Local)
                .map_err(IsolateError::FindIsolateBranchFailed)?;
            isolate_branch
                .delete()
                .map_err(IsolateError::DeleteIsolateBranchFailed)?;

            let repo_root_path =
                paths::repo_root_path(&repo).map_err(IsolateError::GetRepoRootPathFailed)?;
            let repo_root_str = repo_root_path.to_str().ok_or(IsolateError::PathNotUtf8)?;

            match hooks::find_hook(repo_root_str, repo_gitdir_str, "isolate_post_cleanup") {
                Ok(hook_path) => {
                    utils::execute(hook_path.to_str().ok_or(IsolateError::PathNotUtf8)?, &[])
                        .map_err(IsolateError::HookExecutionFailed)?
                }
                Err(hooks::FindHookError::NotFound) => utils::print_warn(
                    color,
                    r#"
  The isolate_post_cleanup hook was not found! Skipping...

  You can find more information and examples of this hook and others at
  the following.

  https://book.git-ps.sh/tool/hooks.html
"#,
                ),
                Err(hooks::FindHookError::NotExecutable(hook_path)) => {
                    let path_str = hook_path.to_str().unwrap_or("unknow path");
                    let msg = format!(
                        r#"
  The isolate_post_cleanup hook was found at

    {}

  but it is NOT executable. Due to this the hook is being skipped. Generally
  this can be corrected with the following.

    chmod u+x {}
"#,
                        path_str, path_str
                    );
                    utils::print_warn(color, &msg);
                }
                Err(e) => return Err(IsolateError::HookNotFound(e)),
            }

            Ok(())
        }
    }
}
