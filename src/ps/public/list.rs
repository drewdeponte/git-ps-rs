use crate::ps::private::config;
use crate::ps::private::list;

use super::super::super::ps;
use super::super::private::git;
use super::super::private::patch_status;
use super::super::private::paths;
use super::super::private::state_management;
use ansi_term::Colour::{Blue, Cyan, Green, Yellow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct RequestReviewRecord {
    patch_stack_id: Uuid,
    branch_name: String,
    commit_id: String,
    published: Option<bool>,
    location_agnostic_hash: Option<String>,
}

#[derive(Debug)]
pub enum ListError {
    RepositoryNotFound,
    GetPatchStackFailed(ps::PatchStackError),
    GetPatchListFailed(ps::GetPatchListError),
    ReadPatchStatesFailed(state_management::ReadPatchStatesError),
    CommitMissing,
    GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    PatchStatusFailed(patch_status::PatchStatusError),
    GetPatchStackBaseTargetFailed,
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    GetHookOutputError(list::ListHookError),
}

pub fn list(color: bool) -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let repo_root_path = paths::repo_root_path(&repo).map_err(ListError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let config =
        config::get_config(repo_root_str, repo_gitdir_str).map_err(ListError::GetConfigFailed)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(ListError::GetPatchStackFailed)?;
    let list_of_patches =
        ps::get_patch_list(&repo, &patch_stack).map_err(ListError::GetPatchListFailed)?;

    let patch_meta_data_path = paths::patch_states_path(&repo);
    let patch_meta_data = state_management::read_patch_states(patch_meta_data_path)
        .map_err(ListError::ReadPatchStatesFailed)?;

    let patch_stack_base_oid = patch_stack
        .base
        .target()
        .ok_or(ListError::GetPatchStackBaseTargetFailed)?;

    let list_of_patches_iter: Box<dyn Iterator<Item = _>> = if config.list.reverse_order {
        Box::new(list_of_patches.into_iter())
    } else {
        Box::new(list_of_patches.into_iter().rev())
    };

    for patch in list_of_patches_iter {
        let mut row = list::ListRow::new(color);
        let commit = repo
            .find_commit(patch.oid)
            .map_err(|_| ListError::CommitMissing)?;
        let patch_state = match ps::commit_ps_id(&commit) {
            Some(ps_id) => patch_meta_data.get(&ps_id),
            None => None,
        };

        let commit_diff_patch_id = git::commit_diff_patch_id(&repo, &commit)
            .map_err(ListError::GetCommitDiffPatchIdFailed)?;
        let patch_status = patch_status::patch_status(
            patch_state,
            &repo,
            commit_diff_patch_id,
            patch_stack_base_oid,
        )
        .map_err(ListError::PatchStatusFailed)?;
        let patch_status_string = patch_status_to_string(patch_status);

        row.add_cell(Some(4), Some(Green), patch.index);
        row.add_cell(Some(6), Some(Cyan), &patch_status_string);

        if config.list.add_extra_patch_info {
            let hook_stdout = list::execute_list_additional_info_hook(
                repo_root_str,
                repo_gitdir_str,
                &[
                    &patch.index.to_string(),
                    &patch_status_string,
                    &patch.oid.to_string(),
                    &patch.summary,
                ],
            )
            .map_err(ListError::GetHookOutputError)?;
            let hook_stdout_len = config.list.extra_patch_info_length;
            row.add_cell(Some(hook_stdout_len), Some(Blue), hook_stdout);
        }

        row.add_cell(Some(7), Some(Yellow), patch.oid);
        row.add_cell(None, None, patch.summary);

        println!("{}", row)
    }

    Ok(())
}

fn patch_status_to_string(patch_status: patch_status::PatchStatus) -> String {
    match patch_status {
        patch_status::PatchStatus::WithoutBranch => "     ",
        patch_status::PatchStatus::BranchCreated => "b    ",
        patch_status::PatchStatus::BranchCreatedButLocalHasChanged => "b+   ",
        patch_status::PatchStatus::PushedToRemote => "s    ",
        patch_status::PatchStatus::PushedToRemoteButLocalHasChanged => "s+   ",
        patch_status::PatchStatus::PushedToRemoteButRemoteHasChanged => "s  ! ",
        patch_status::PatchStatus::PushedToRemoteButBothHaveChanged => "s+ ! ",
        patch_status::PatchStatus::PushedToRemoteNowBehind => "s   ↓",
        patch_status::PatchStatus::PushedToRemoteNowBehindButLocalHasChanged => "s+  ↓",
        patch_status::PatchStatus::RequestedReview => "rr   ",
        patch_status::PatchStatus::RequestedReviewButLocalHasChanged => "rr+  ",
        patch_status::PatchStatus::RequestedReviewButRemoteHasChanged => "rr ! ",
        patch_status::PatchStatus::RequestedReviewButBothHaveChanged => "rr+! ",
        patch_status::PatchStatus::RequestedReviewNowBehind => "rr  ↓",
        patch_status::PatchStatus::RequestedReviewNowBehindButLocalHasChanged => "rr+ ↓",
        patch_status::PatchStatus::Integrated => "int  ",
    }
    .to_string()
}
