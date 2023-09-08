use super::super::super::ps;
use super::super::private::cherry_picking;
use super::super::private::config;
use super::super::private::git;
use super::super::private::paths;
use super::verify_isolation;
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
    FindCommitFailed(git2::Error),
    GetCommitParentZeroFailed(git2::Error),
    CherryPickFailed(git::GitError),
    OpenGitConfigFailed(git2::Error),
    PatchCommitNotFound(git2::Error),
    PatchSummaryMissing,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteBranchNameFailed,
    PushFailed(git::ExtForcePushError),
    FailedToMapIndexesForCherryPick(cherry_picking::MapRangeForCherryPickError),
    IsolationVerificationFailed(verify_isolation::VerifyIsolationError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
}

pub fn branch(
    start_patch_index: usize,
    end_patch_index_option: Option<usize>,
    given_branch_name_option: Option<String>,
    push_to_remote: bool,
    color: bool,
) -> Result<(), BranchError> {
    let repo = git::create_cwd_repo().map_err(BranchError::OpenRepositoryFailed)?;
    let git_config = git2::Config::open_default().map_err(BranchError::OpenGitConfigFailed)?;

    let repo_root_path =
        paths::repo_root_path(&repo).map_err(BranchError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(BranchError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(BranchError::PathNotUtf8)?;
    let config =
        config::get_config(repo_root_str, repo_gitdir_str).map_err(BranchError::GetConfigFailed)?;

    // find the base of the current patch stack
    let patch_stack = ps::get_patch_stack(&repo).map_err(BranchError::GetPatchStackFailed)?;
    let patch_stack_base_commit = patch_stack
        .base
        .peel_to_commit()
        .map_err(|_| BranchError::PatchStackBaseNotFound)?;

    let patches_vec =
        ps::get_patch_list(&repo, &patch_stack).map_err(BranchError::GetPatchListFailed)?;

    if config.branch.verify_isolation {
        verify_isolation::verify_isolation(start_patch_index, end_patch_index_option, color)
            .map_err(BranchError::IsolationVerificationFailed)?;
    }

    let start_patch_oid = patches_vec
        .get(start_patch_index)
        .ok_or(BranchError::PatchIndexNotFound)?
        .oid;
    let start_patch_commit = repo
        .find_commit(start_patch_oid)
        .map_err(BranchError::FindCommitFailed)?;

    // generate the default branch name and use provided branch name or fallback
    let patch_summary = start_patch_commit
        .summary()
        .ok_or(BranchError::PatchSummaryMissing)?;
    let default_branch_name = ps::generate_branch_branch_name(patch_summary);
    let branch_name = given_branch_name_option.unwrap_or(default_branch_name);

    // create a branch on the base of the current patch stack
    let branch = repo
        .branch(branch_name.as_str(), &patch_stack_base_commit, true)
        .map_err(BranchError::CreateBranchFailed)?;
    let branch_ref_name = branch.get().name().ok_or(BranchError::BranchNameNotUtf8)?;

    // cherry pick the patch or patch range onto new isolation branch
    let cherry_pick_range = cherry_picking::map_range_for_cherry_pick(
        &patches_vec,
        start_patch_index,
        end_patch_index_option,
    )
    .map_err(BranchError::FailedToMapIndexesForCherryPick)?;

    git::cherry_pick(
        &repo,
        &git_config,
        cherry_pick_range.root_oid,
        cherry_pick_range.leaf_oid,
        branch_ref_name,
    )
    .map_err(BranchError::CherryPickFailed)?;

    // push branch up to remote branch
    if push_to_remote || config.branch.push_to_remote {
        // get remote name of current branch (e.g. origin)
        let cur_branch_name =
            git::get_current_branch(&repo).ok_or(BranchError::CurrentBranchNameMissing)?;
        let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str())
            .map_err(|_| BranchError::GetUpstreamBranchNameFailed)?;
        let remote_name = repo
            .branch_remote_name(&branch_upstream_name)
            .map_err(|_| BranchError::GetRemoteBranchNameFailed)?;

        git::ext_push(
            true,
            remote_name.as_str().unwrap(),
            branch_ref_name,
            branch_ref_name,
        )
        .map_err(BranchError::PushFailed)?;
    }

    Ok(())
}
