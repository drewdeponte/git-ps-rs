use super::super::super::ps;
use super::super::private::commit_is_behind;
use super::super::private::config;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::state_computation;
use super::super::private::utils;
use super::super::public::pull;
use super::super::public::show;
use super::super::public::sync;
use super::verify_isolation;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub enum IntegrateError {
    RepositoryNotFound,
    FindPatchCommitFailed(ps::FindPatchCommitError),
    CommitPsIdMissing,
    PatchMetaDataMissing,
    PatchHasNotBeenPushed,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    GetHeadFailed(git2::Error),
    HeadMissingTarget,
    ConvertStringToStrFailed,
    FindRemoteRrBranchFailed(git2::Error),
    RemoteRrBranchMissingTarget,
    CommonAncestorFailed(git::CommonAncestorError),
    GetRevWalkerFailed(git::GitError),
    FindRrBranchCommitFailed(git2::Error),
    RrBranchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    PatchesDiffer,
    PushFailed(ps::private::git::ExtForcePushError),
    DeleteLocalBranchFailed(git2::Error),
    DeleteRemoteBranchFailed(git::ExtDeleteRemoteBranchError),
    BranchOperationFailed(ps::private::request_review_branch::RequestReviewBranchError),
    GetBranchNameFailed(git2::Error),
    CreatedBranchMissingName,
    SingularCommitOfBranchError(git::SingularCommitOfBranchError),
    UpdateLocalRequestReviewBranchFailed(
        ps::private::request_review_branch::RequestReviewBranchError,
    ),
    FetchFailed(git::ExtFetchError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    UserVerificationFailed(GetVerificationError),
    ShowFailed(show::ShowError),
    GetPatchStackFailed(ps::PatchStackError),
    GetCommitIsBehindFailed(commit_is_behind::CommitIsBehindError),
    PatchIsBehind,
    PullFailed(pull::PullError),
    HookExecutionFailed(utils::ExecuteError),
    HookNotFound(hooks::FindHookError),
    SyncFailed(sync::SyncError),
    PatchStackBaseNotFound,
    PatchStackHeadNoName,
    GetListPatchInfoFailed(state_computation::GetListPatchInfoError),
    PatchBranchAmbiguous,
    PatchHasNoAssociatedBranch,
    PatchBranchNotSingularCommit,
    PatchHasNoAssociatedUpstreamBranch,
    PatchUpstreamBranchNotSingularCommit,
    AssociatedBranchPatchAndUpstreamBranchPatchMismatch,
    PatchAndAssociatedBranchPatchMismatch,
    FindAssociatedBranchFailed(git2::Error),
}

pub fn integrate(
    patch_index: usize,
    force: bool,
    keep_branch: bool,
    given_branch_name_option: Option<String>,
    color: bool,
) -> Result<(), IntegrateError> {
    let repo = git::create_cwd_repo().map_err(|_| IntegrateError::RepositoryNotFound)?;

    // verify that the patch-index has a corresponding commit
    let patch_commit =
        ps::find_patch_commit(&repo, patch_index).map_err(IntegrateError::FindPatchCommitFailed)?;

    let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit)
        .map_err(IntegrateError::PatchCommitDiffPatchIdFailed)?;

    let repo_root_path =
        paths::repo_root_path(&repo).map_err(IntegrateError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(IntegrateError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path
        .to_str()
        .ok_or(IntegrateError::PathNotUtf8)?;

    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(IntegrateError::GetConfigFailed)?;

    if force {
        // force
        //  x prompt for reassurance (based on config)
        //  x fetch to get new remote state
        //  x create/replace the local request review branch
        //  x verify isolation (based on config)
        //  x publish the patch from the local patch branch up to the patch stack uptstream
        //  x execute the integrate_post_push hook
        //  x delete local rr branch (based on command line option)

        // prompt for reassurance
        if config.integrate.prompt_for_reassurance {
            match show::show(patch_index) {
                Err(show::ShowError::ExitSignal(13)) => utils::print_warn(
                    color,
                    r#"
  Warning: showing the patch exited with a SIGPIPE. This is likely because you
  exited the pager (e.g. less) without going to the last page.

  See https://github.com/uptech/git-ps-rs/issues/120 for details on why this
  happens.
"#,
                ),
                Err(e) => return Err(IntegrateError::ShowFailed(e)),
                Ok(_) => (),
            }
            get_verification().map_err(IntegrateError::UserVerificationFailed)?;
        }

        // fetch so we get new remote state
        git::ext_fetch().map_err(IntegrateError::FetchFailed)?;

        // create/replace the request review branch
        let (patch_branch, ps_id, new_commit_oid) =
            ps::private::request_review_branch::request_review_branch(
                &repo,
                patch_index,
                given_branch_name_option,
            )
            .map_err(IntegrateError::BranchOperationFailed)?;

        let patch_branch_name = patch_branch
            .name()
            .map_err(IntegrateError::GetBranchNameFailed)?
            .ok_or(IntegrateError::CreatedBranchMissingName)?;

        let cur_patch_stack_branch_name =
            git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
        let cur_patch_stack_branch_upstream_name =
            git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
                .map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
        let cur_patch_stack_remote_name = repo
            .branch_remote_name(&cur_patch_stack_branch_upstream_name)
            .map_err(|_| IntegrateError::GetRemoteNameFailed)?;
        let cur_patch_stack_remote_name_str = cur_patch_stack_remote_name
            .as_str()
            .ok_or(IntegrateError::ConvertStringToStrFailed)?;

        // verify isolation
        if config.integrate.verify_isolation {
            verify_isolation::verify_isolation(patch_index, None, color)
                .map_err(IntegrateError::IsolationVerificationFailed)?;
        }

        let pattern = format!("refs/remotes/{}/", cur_patch_stack_remote_name_str);
        let cur_patch_stack_upstream_branch_shorthand =
            str::replace(&cur_patch_stack_branch_upstream_name, pattern.as_str(), "");

        // publish the patch from the local patch branch up to the patch stack uptstream
        // e.g. git push origin ps/rr/whatever-branch:main
        git::ext_push(
            false,
            cur_patch_stack_remote_name_str,
            patch_branch_name,
            &cur_patch_stack_upstream_branch_shorthand,
        )
        .map_err(IntegrateError::PushFailed)?;

        // execute the integrate_post_push hook
        match hooks::find_hook(repo_root_str, repo_gitdir_str, "integrate_post_push") {
            Ok(hook_path) => utils::execute(
                hook_path.to_str().ok_or(IntegrateError::PathNotUtf8)?,
                &[&format!("{}", new_commit_oid)],
            )
            .map_err(IntegrateError::HookExecutionFailed)?,
            Err(hooks::FindHookError::NotFound) => integrate_post_push_hook_missing(color),
            Err(hooks::FindHookError::NotExecutable(hook_path)) => {
                integrate_post_push_hook_not_executable(
                    color,
                    hook_path.to_str().unwrap_or("unknow path"),
                )
            }
            Err(e) => return Err(IntegrateError::HookNotFound(e)),
        }

        // clean up the local rr branch
        if !keep_branch {
            //  - TODO: in the force case, make the delete also delete the remote branch if exists
            let mut local_branch = repo
                .find_branch(patch_branch_name, git2::BranchType::Local)
                .map_err(IntegrateError::DeleteLocalBranchFailed)?;
            local_branch
                .delete()
                .map_err(IntegrateError::DeleteLocalBranchFailed)?;
        }
    } else {
        // non-forced
        //  x prompt for reassurance (based on config)
        //  x verify that the commit has a patch stack id
        //  x fetch to get new remote state
        //  x make sure that the upstream patch branch has a singular commit
        //  x verify that the remote rr branche's patche's diff hash matches that of the local patch in the patch stack
        //  x verify that the patch stack upstream base hasn't left the remote patch behind
        //  x verify isolation (based on config)
        //  x publish patch from remote branch up to the patch stack upstream (e.g. git push origin origin/ps/rr/whatever-branch:main)
        //  x execute the integrate_post_push hook
        //  x delete local & remote rr branch (based on command line option)

        // prompt for reassurance
        if config.integrate.prompt_for_reassurance {
            match show::show(patch_index) {
                Err(show::ShowError::ExitSignal(13)) => utils::print_warn(
                    color,
                    r#"
  Warning: showing the patch exited with a SIGPIPE. This is likely because you
  exited the pager (e.g. less) without going to the last page.

  See https://github.com/uptech/git-ps-rs/issues/120 for details on why this
  happens.
"#,
                ),
                Err(e) => return Err(IntegrateError::ShowFailed(e)),
                Ok(_) => (),
            }
            get_verification().map_err(IntegrateError::UserVerificationFailed)?;
        }

        // verify that the commit has a patch stack id
        let ps_id = ps::commit_ps_id(&patch_commit).ok_or(IntegrateError::CommitPsIdMissing)?;

        // fetch so we get new remote state
        git::ext_fetch().map_err(IntegrateError::FetchFailed)?;

        let patch_stack =
            ps::get_patch_stack(&repo).map_err(IntegrateError::GetPatchStackFailed)?;

        // fetch computed state from Git tree
        let patch_stack_base_commit = patch_stack
            .base
            .peel_to_commit()
            .map_err(|_| IntegrateError::PatchStackBaseNotFound)?;

        let head_ref_name = patch_stack
            .head
            .shorthand()
            .ok_or(IntegrateError::PatchStackHeadNoName)?;

        let patch_info_collection: HashMap<Uuid, state_computation::PatchGitInfo> =
            state_computation::get_list_patch_info(
                &repo,
                patch_stack_base_commit.id(),
                head_ref_name,
            )
            .map_err(IntegrateError::GetListPatchInfoFailed)?;

        // get the associated branch name, or error
        let patch_associated_branch_info: state_computation::ListBranchInfo =
            match patch_info_collection.get(&ps_id) {
                Some(patch_info) => {
                    if patch_info.branches.len() == 1 {
                        Ok(patch_info.branches.first().unwrap().clone())
                    } else {
                        Err(IntegrateError::PatchBranchAmbiguous)
                    }
                }
                None => Err(IntegrateError::PatchHasNoAssociatedBranch),
            }?;

        //  - make sure that the upstream patch branch has a singular commit
        if patch_associated_branch_info.commit_count != 1 {
            return Err(IntegrateError::PatchBranchNotSingularCommit);
        }
        let patch_associated_upstream_branch_info: state_computation::ListUpstreamBranchInfo =
            match patch_associated_branch_info.upstream {
                Some(patch_upstream_branch_info) => {
                    if patch_upstream_branch_info.commit_count == 1 {
                        Ok(patch_upstream_branch_info.clone())
                    } else {
                        Err(IntegrateError::PatchUpstreamBranchNotSingularCommit)
                    }
                }
                None => Err(IntegrateError::PatchHasNoAssociatedUpstreamBranch),
            }?;

        // - verify the patch diffs match
        if patch_associated_branch_info
            .patches
            .first()
            .unwrap()
            .commit_diff_id
            != patch_associated_upstream_branch_info
                .patches
                .first()
                .unwrap()
                .commit_diff_id
        {
            return Err(IntegrateError::AssociatedBranchPatchAndUpstreamBranchPatchMismatch);
        }

        if patch_commit_diff_patch_id
            != patch_associated_branch_info
                .patches
                .first()
                .unwrap()
                .commit_diff_id
        {
            return Err(IntegrateError::PatchAndAssociatedBranchPatchMismatch);
        }

        //  - verify that upstream base hasn't left the remote patch behind
        let patch_associated_upstream_branch = repo
            .find_branch(
                &patch_associated_upstream_branch_info.name,
                git2::BranchType::Remote,
            )
            .map_err(IntegrateError::FindAssociatedBranchFailed)?;
        let patch_associated_upstream_branch_oid: git2::Oid =
            patch_associated_upstream_branch.get().target().unwrap();

        let common_ancestor_oid = git::common_ancestor(
            &repo,
            patch_associated_upstream_branch_oid,
            patch_stack_base_commit.id(),
        )
        .map_err(IntegrateError::CommonAncestorFailed)?;

        if common_ancestor_oid != patch_stack_base_commit.id() {
            // patch stack base has left the remote patch behind
            return Err(IntegrateError::PatchIsBehind);
        }

        // verify isolation
        if config.integrate.verify_isolation {
            verify_isolation::verify_isolation(patch_index, None, color)
                .map_err(IntegrateError::IsolationVerificationFailed)?;
        }

        //  - publish patch from remote branch up to the patch stack upstream (e.g. git push origin origin/ps/rr/whatever-branch:main)
        // At this point we are pretty confident that things are properly in sync
        // and therefore we allow the actual act of integrating into to upstream
        // happen.
        // e.g. git push origin origin/ps/rr/whatever-branch:main
        let cur_patch_stack_branch_name =
            git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
        let cur_patch_stack_branch_upstream_name =
            git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
                .map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
        let cur_patch_stack_remote_name = repo
            .branch_remote_name(&cur_patch_stack_branch_upstream_name)
            .map_err(|_| IntegrateError::GetRemoteNameFailed)?;
        let cur_patch_stack_remote_name_str = cur_patch_stack_remote_name
            .as_str()
            .ok_or(IntegrateError::ConvertStringToStrFailed)?;

        println!("remote: {}", cur_patch_stack_remote_name_str);
        println!(
            "src refspec: {}",
            &patch_associated_upstream_branch_info.reference
        );
        println!("dst refspec: {}", &cur_patch_stack_branch_upstream_name);

        git::ext_push(
            false,
            cur_patch_stack_remote_name_str,
            &patch_associated_upstream_branch_info.reference,
            &cur_patch_stack_branch_upstream_name,
        )
        .map_err(IntegrateError::PushFailed)?;

        //  - execute the integrate_post_push hook
        match hooks::find_hook(repo_root_str, repo_gitdir_str, "integrate_post_push") {
            Ok(hook_path) => utils::execute(
                hook_path.to_str().ok_or(IntegrateError::PathNotUtf8)?,
                &[&format!("{}", patch_associated_upstream_branch_oid)],
            )
            .map_err(IntegrateError::HookExecutionFailed)?,
            Err(hooks::FindHookError::NotFound) => integrate_post_push_hook_missing(color),
            Err(hooks::FindHookError::NotExecutable(hook_path)) => {
                integrate_post_push_hook_not_executable(
                    color,
                    hook_path.to_str().unwrap_or("unknow path"),
                )
            }
            Err(e) => return Err(IntegrateError::HookNotFound(e)),
        }

        //  - delete local & remote rr branch (based on command line option)
        if !keep_branch {
            let mut local_branch = repo
                .find_branch(&patch_associated_branch_info.name, git2::BranchType::Local)
                .map_err(IntegrateError::DeleteLocalBranchFailed)?;
            local_branch
                .delete()
                .map_err(IntegrateError::DeleteLocalBranchFailed)?;

            git::ext_delete_remote_branch(
                &patch_associated_upstream_branch_info.remote,
                &patch_associated_upstream_branch_info.name,
            )
            .map_err(IntegrateError::DeleteRemoteBranchFailed)?;
        }
    }

    if config.integrate.pull_after_integrate {
        pull::pull(color).map_err(IntegrateError::PullFailed)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum GetVerificationError {
    ReadLineFailed(std::io::Error),
    UserRejected(String),
}

fn get_verification() -> Result<(), GetVerificationError> {
    let mut answer = String::new();
    println!("\n\nAre you sure you want to integrate this patch? (y/N)");
    std::io::stdin()
        .read_line(&mut answer)
        .map_err(GetVerificationError::ReadLineFailed)?;
    let normalized_answer = answer.to_lowercase().trim().to_string();
    if normalized_answer == "yes" || normalized_answer == "y" {
        Ok(())
    } else {
        Err(GetVerificationError::UserRejected(normalized_answer))
    }
}

fn integrate_post_push_hook_missing(color: bool) {
    utils::print_warn(
        color,
        r#"
The integrate_post_push hook was not found, therefore skipping.

You can find more information and examples of this hook and others at
the following.

https://book.git-ps.sh/tool/hooks.html
"#,
    );
}

fn integrate_post_push_hook_not_executable(color: bool, hook_path: &str) {
    let msg = format!(
        r#"
The integrate_post_push hook was found at

  {}

but it is NOT executable. Due to this the hook is being skipped. Generally
this can be corrected with the following.

  chmod u+x {}
"#,
        hook_path, hook_path
    );
    utils::print_warn(color, &msg);
}
