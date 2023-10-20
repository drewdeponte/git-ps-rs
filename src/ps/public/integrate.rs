use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::state_computation;
use super::super::private::utils;
use super::super::public::pull;
use super::super::public::show;
use super::verify_isolation;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub enum IntegrateError {
    RepositoryNotFound,
    GetPatchStackFailed(ps::PatchStackError),
    GetPatchListFailed(ps::GetPatchListError),
    PatchIndexRangeOutOfBounds(ps::PatchRangeWithinStackBoundsError),
    OpenGitConfigFailed(git2::Error),
    AddPatchIdsFailed(ps::AddPatchIdsError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    ShowFailed(show::ShowError),
    UserVerificationFailed(GetVerificationError),
    FetchFailed(git::ExtFetchError),
    PatchStackBaseNotFound,
    PatchStackHeadNoName,
    GetListPatchInfoFailed(state_computation::GetListPatchInfoError),
    HasNoAssociatedBranch,
    AssociatedBranchAmbiguous,
    FindPatchCommitFailed(git2::Error),
    MissingPatchId,
    MissingPatchInfo,
    UpstreamBranchInfoMissing,
    CommitCountMissmatch(usize, usize),
    PatchAndRemotePatchIdMissmatch(usize),
    PatchDiffHashMissmatch(usize),
    PatchMissingDiffHash,
    CreateOrReplaceBranchFailed(ps::private::branch::BranchError),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    GetPatchBranchNameFailed(git2::Error),
    CreatedBranchMissingName,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    ConvertStringToStrFailed,
    PushFailed(ps::private::git::ExtForcePushError),
    HookExecutionFailed(utils::ExecuteError),
    VerifyHookExecutionFailed(utils::ExecuteError),
    HookNotFound(hooks::FindHookError),
    FindPatchBranchFailed(git2::Error),
    GetBranchUpstreamRemoteFailed(git2::Error),
    BranchUpstreamRemoteNotValidUtf8,
    RemoteRrBranchNameMissing,
    DeleteRemoteBranchFailed(git::ExtDeleteRemoteBranchError),
    DeleteLocalBranchFailed(git2::Error),
    PullFailed(pull::PullError),
    FindRemoteFailed(git2::Error),
    RemoteUrlNotUtf8,
}

pub fn integrate(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    force: bool,
    keep_branch: bool,
    given_branch_name_option: Option<String>,
    color: bool,
) -> Result<(), IntegrateError> {
    // x validate patch indexes are within bounds
    // x add patch ids
    // x prompt_for_reassurance (based on config)
    // x git fetch - update the knowledge of the remotes
    // - if NOT force
    //     x figure out associated branch
    //     x verify has associated branch, exit with error
    //     x check to make sure patches match between stack & remote
    //     - execute hook to verify PR approval & CI status
    // x create/replace the request review branch
    //     - in fresh case, it creates the branch, in existing case it updates it to the latest state from ps
    // x verify isolation (verifies cherry-pick cleanly but also verify isolation hook passes)
    // x publish the patch(es) from local patch branch up to patch stack upstream
    // x execute integrate post push hook
    // x optionally (based on config) delete remote if exists request review branch
    // x optionally (based on config) delete local request review branch
    // x optionnaly pull (based on config)

    let repo = git::create_cwd_repo().map_err(|_| IntegrateError::RepositoryNotFound)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(IntegrateError::GetPatchStackFailed)?;
    let patches_vec =
        ps::get_patch_list(&repo, &patch_stack).map_err(IntegrateError::GetPatchListFailed)?;

    // validate patch indexes are within bounds
    ps::patch_range_within_stack_bounds(start_patch_index, end_patch_index, &patches_vec)
        .map_err(IntegrateError::PatchIndexRangeOutOfBounds)?;

    // add patch ids to commits in patch stack missing them
    let git_config = git2::Config::open_default().map_err(IntegrateError::OpenGitConfigFailed)?;
    ps::add_patch_ids(&repo, &git_config).map_err(IntegrateError::AddPatchIdsFailed)?;

    let repo_root_path =
        paths::repo_root_path(&repo).map_err(IntegrateError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(IntegrateError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path
        .to_str()
        .ok_or(IntegrateError::PathNotUtf8)?;

    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(IntegrateError::GetConfigFailed)?;

    // prompt for reassurance
    if config.integrate.prompt_for_reassurance {
        match show::show(start_patch_index, end_patch_index) {
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

    // compute the state from git
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
        state_computation::get_list_patch_info(&repo, patch_stack_base_commit.id(), head_ref_name)
            .map_err(IntegrateError::GetListPatchInfoFailed)?;

    // figure out the associated branch
    let range_patch_branches = ps::patch_series_unique_branch_names(
        &repo,
        &patches_vec,
        &patch_info_collection,
        start_patch_index,
        end_patch_index,
    );

    if !force {
        // verify has associated branch, exit with error
        if range_patch_branches.is_empty() {
            return Err(IntegrateError::HasNoAssociatedBranch);
        } else if range_patch_branches.len() > 1 && given_branch_name_option.is_none() {
            return Err(IntegrateError::AssociatedBranchAmbiguous);
        }

        let patch_associated_branch_name = match given_branch_name_option {
            Some(ref bn) => bn.clone(),
            None => range_patch_branches.first().unwrap().to_string(),
        };

        // check to make sure patches match between stack & remote

        // get a patch id of any patch in series
        let some_patches_basic_info = patches_vec.get(start_patch_index).unwrap();
        let some_patch_commit = repo
            .find_commit(some_patches_basic_info.oid)
            .map_err(IntegrateError::FindPatchCommitFailed)?;
        let some_patch_id =
            ps::commit_ps_id(&some_patch_commit).ok_or(IntegrateError::MissingPatchId)?;

        let some_patch_info_option = patch_info_collection.get(&some_patch_id);
        if some_patch_info_option.is_none() {
            return Err(IntegrateError::MissingPatchInfo);
        }

        let mut branches_iter = some_patch_info_option.unwrap().branches.iter();
        let upstream_branch_info = branches_iter
            .find(|b| b.name == patch_associated_branch_name)
            .and_then(|lbi| lbi.upstream.as_ref());
        let upstream_branch_info_ref = upstream_branch_info.as_ref();

        let patch_series_indexes = match end_patch_index {
            Some(end_index) => (start_patch_index..=end_index).collect(),
            None => vec![start_patch_index],
        };

        if upstream_branch_info_ref.is_none() {
            return Err(IntegrateError::UpstreamBranchInfoMissing);
        }

        if upstream_branch_info_ref.unwrap().commit_count != patch_series_indexes.len() {
            return Err(IntegrateError::CommitCountMissmatch(
                patch_series_indexes.len(),
                upstream_branch_info_ref.unwrap().commit_count,
            ));
        }

        // get commits from patch stack for the patch series
        let patch_series_commits: Vec<git2::Commit> = patch_series_indexes
            .iter()
            .map(|i| patches_vec.get(*i).unwrap())
            .map(|pi| repo.find_commit(pi.oid).unwrap())
            .collect();

        for (idx, patch_series_commit) in patch_series_commits.iter().enumerate() {
            let remote_patch_info = upstream_branch_info_ref.unwrap().patches.get(idx).unwrap();

            match ps::commit_ps_id(patch_series_commit) {
                None => return Err(IntegrateError::MissingPatchId),
                Some(patch_id) => {
                    if patch_id != remote_patch_info.patch_id {
                        return Err(IntegrateError::PatchAndRemotePatchIdMissmatch(
                            start_patch_index + idx,
                        ));
                    }
                }
            }

            match git::commit_diff_patch_id(&repo, patch_series_commit) {
                Ok(patch_diff_hash) => {
                    if patch_diff_hash != remote_patch_info.commit_diff_id {
                        return Err(IntegrateError::PatchDiffHashMissmatch(
                            start_patch_index + idx,
                        ));
                    }
                }
                Err(_) => {
                    return Err(IntegrateError::PatchMissingDiffHash);
                }
            }
        }

        // execute hook to verify PR approval & CI status
        let cur_patch_stack_branch_name =
            git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
        let cur_patch_stack_upstream_branch_name =
            git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
                .map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
        let cur_patch_stack_remote_name = repo
            .branch_remote_name(&cur_patch_stack_upstream_branch_name)
            .map_err(|_| IntegrateError::GetRemoteNameFailed)?;
        let cur_patch_stack_upstream_branch_remote_name_str = cur_patch_stack_remote_name
            .as_str()
            .ok_or(IntegrateError::ConvertStringToStrFailed)?;
        let cur_patch_stack_upstream_branch_remote = repo
            .find_remote(cur_patch_stack_upstream_branch_remote_name_str)
            .map_err(IntegrateError::FindRemoteFailed)?;
        let cur_patch_stack_upstream_branch_remote_url_str = cur_patch_stack_upstream_branch_remote
            .url()
            .ok_or(IntegrateError::RemoteUrlNotUtf8)?;

        let pattern = format!(
            "refs/remotes/{}/",
            cur_patch_stack_upstream_branch_remote_name_str
        );
        let cur_patch_stack_upstream_branch_name_relative_to_remote =
            str::replace(&cur_patch_stack_upstream_branch_name, pattern.as_str(), "");

        match hooks::find_hook(repo_root_str, repo_gitdir_str, "integrate_verify") {
            Ok(hook_path) => utils::execute(
                hook_path.to_str().ok_or(IntegrateError::PathNotUtf8)?,
                &[
                    &patch_associated_branch_name,
                    &cur_patch_stack_upstream_branch_name_relative_to_remote,
                    cur_patch_stack_upstream_branch_remote_name_str,
                    cur_patch_stack_upstream_branch_remote_url_str,
                ],
            )
            .map_err(IntegrateError::VerifyHookExecutionFailed)?,
            Err(hooks::FindHookError::NotFound) => {}
            Err(hooks::FindHookError::NotExecutable(hook_path)) => {
                integrate_verify_hook_not_executable(
                    color,
                    hook_path.to_str().unwrap_or("unknow path"),
                )
            }
            Err(e) => return Err(IntegrateError::HookNotFound(e)),
        }
    }

    // create/replace the request review branch
    let (patch_branch, new_commit_oid) = ps::private::branch::branch(
        &repo,
        start_patch_index,
        end_patch_index,
        given_branch_name_option,
    )
    .map_err(IntegrateError::CreateOrReplaceBranchFailed)?;

    // verify isolation
    if config.integrate.verify_isolation {
        verify_isolation::verify_isolation(start_patch_index, end_patch_index, color)
            .map_err(IntegrateError::IsolationVerificationFailed)?;
    }

    // publish the patch(es) from local patch branch up to patch stack upstream
    let patch_branch_name = patch_branch
        .name()
        .map_err(IntegrateError::GetPatchBranchNameFailed)?
        .ok_or(IntegrateError::CreatedBranchMissingName)?;
    let patch_branch_ref_name = patch_branch.get().name().unwrap();

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
        Err(hooks::FindHookError::NotFound) => {}
        Err(hooks::FindHookError::NotExecutable(hook_path)) => {
            integrate_post_push_hook_not_executable(
                color,
                hook_path.to_str().unwrap_or("unknow path"),
            )
        }
        Err(e) => return Err(IntegrateError::HookNotFound(e)),
    }

    //  delete local & remote rr branch (based on command line option)
    if !keep_branch {
        let mut local_branch = repo
            .find_branch(patch_branch_name, git2::BranchType::Local)
            .map_err(IntegrateError::FindPatchBranchFailed)?;

        // if we have a remote tracking branch, delete it
        if let Ok(remote_branch) = local_branch.upstream() {
            let remote_branch_remote = repo
                .branch_upstream_remote(patch_branch_ref_name)
                .map_err(IntegrateError::GetBranchUpstreamRemoteFailed)?;
            let remote_branch_remote_str = remote_branch_remote
                .as_str()
                .ok_or(IntegrateError::BranchUpstreamRemoteNotValidUtf8)?;

            let remote_branch_name = remote_branch
                .name()
                .map_err(IntegrateError::GetPatchBranchNameFailed)?
                .ok_or(IntegrateError::RemoteRrBranchNameMissing)?;

            let pattern = format!("{}/", remote_branch_remote_str);
            let patch_associated_upstream_branch_name_relative_to_remote =
                str::replace(remote_branch_name, &pattern, "");

            git::ext_delete_remote_branch(
                remote_branch_remote_str,
                &patch_associated_upstream_branch_name_relative_to_remote,
            )
            .map_err(IntegrateError::DeleteRemoteBranchFailed)?;
        }

        // now that we have deleted the remote, lets delete the local
        local_branch
            .delete()
            .map_err(IntegrateError::DeleteLocalBranchFailed)?;
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

fn integrate_verify_hook_not_executable(color: bool, hook_path: &str) {
    let msg = format!(
        r#"
The integrate_verify hook was found at

  {}

but it is NOT executable. Due to this the hook is being skipped. Generally
this can be corrected with the following.

  chmod u+x {}
"#,
        hook_path, hook_path
    );
    utils::print_warn(color, &msg);
}
