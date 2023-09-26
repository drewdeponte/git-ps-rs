use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::paths;
use super::super::private::state_computation;
use crate::ps::private::list;
use ansi_term::Colour::{Blue, Cyan, Green, Yellow};
use std::cmp::Ordering;

#[derive(Debug)]
pub enum ListError {
    RepositoryNotFound,
    GetPatchStackFailed(ps::PatchStackError),
    GetPatchListFailed(ps::GetPatchListError),
    GetRepoRootPathFailed(paths::PathsError),
    PathNotUtf8,
    GetConfigFailed(config::GetConfigError),
    GetCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    GetHookOutputError(list::ListHookError),
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
}

pub fn list(color: bool) -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let repo_root_path = paths::repo_root_path(&repo).map_err(ListError::GetRepoRootPathFailed)?;
    let repo_root_str = repo_root_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let config =
        config::get_config(repo_root_str, repo_gitdir_str).map_err(ListError::GetConfigFailed)?;

    let cur_patch_stack_branch_ref =
        git::get_current_branch(&repo).ok_or(ListError::CurrentBranchNameMissing)?;
    let cur_patch_stack_branch_upstream_ref =
        git::branch_upstream_name(&repo, &cur_patch_stack_branch_ref)
            .map_err(|_| ListError::GetUpstreamBranchNameFailed)?;
    let cur_patch_stack_branch_name = str::replace(&cur_patch_stack_branch_ref, "refs/heads/", "");
    let cur_patch_stack_branch_upstream_name =
        str::replace(&cur_patch_stack_branch_upstream_ref, "refs/remotes/", "");

    // We do know what branch we are currently checked out on when running this command. It seems
    // like we should use that as the base branch.

    let patch_stack = ps::get_patch_stack(&repo).map_err(ListError::GetPatchStackFailed)?;

    let list_of_patches =
        ps::get_patch_list(&repo, &patch_stack).map_err(ListError::GetPatchListFailed)?;

    let base_oid = patch_stack.base.target().unwrap();

    let patch_info_collection =
        state_computation::get_list_patch_info(&repo, base_oid, &cur_patch_stack_branch_name)
            .unwrap();

    println!(
        "Stack {} following {}\n",
        &cur_patch_stack_branch_name, &cur_patch_stack_branch_upstream_name
    );

    let list_of_patches_iter: Box<dyn Iterator<Item = _>> = if config.list.reverse_order {
        Box::new(list_of_patches.into_iter())
    } else {
        Box::new(list_of_patches.into_iter().rev())
    };

    for patch in list_of_patches_iter {
        let mut row = list::ListRow::new(color);
        let commit = repo.find_commit(patch.oid).unwrap();

        let commit_diff_id = git::commit_diff_patch_id(&repo, &commit)
            .map_err(ListError::GetCommitDiffPatchIdFailed)?;

        row.add_cell(Some(4), Some(Green), patch.index);
        row.add_cell(Some(7), Some(Yellow), patch.oid);
        row.add_cell(Some(50), None, patch.summary.clone());

        if let Some(ps_id) = ps::commit_ps_id(&commit) {
            if let Some(patch_info) = patch_info_collection.get(&ps_id) {
                row.add_cell(Some(1), Some(Cyan), "(");
                for b in patch_info.branches.iter() {
                    match patch_info.branches.len().cmp(&1) {
                        Ordering::Greater => {
                            row.add_cell(None, None, b.name.clone());
                        }
                        Ordering::Less => {}
                        Ordering::Equal => {
                            let branch_info = patch_info.branches.first().unwrap();
                            if !branch_info.name.starts_with("ps/rr/") {
                                row.add_cell(None, None, b.name.clone());
                            }
                        }
                    }

                    // Decided that we need to make the request review branches tracking branches
                    // because when we do an rr or other similar commands we have to find the
                    // associated branch(es). In the case where it is a single branch we can assume
                    // that branch and then use it's associated tracking branch. In the case where
                    // multiple branches exist with the patch the user will have to select a
                    // branch somehow and then once they select a branch then we can use the
                    // tracking branch of that branch to know where to push changes.

                    let mut state_string = String::new();

                    let branch_patch: state_computation::PatchInfo = b
                        .patches
                        .iter()
                        .filter(|p| p.patch_id == ps_id)
                        .map(|p| p.to_owned())
                        .collect::<Vec<state_computation::PatchInfo>>()
                        .first()
                        .unwrap()
                        .clone();
                    state_string.push('l');

                    if branch_patch.commit_diff_id != commit_diff_id {
                        state_string.push('*');
                    }

                    if b.patches.len() < b.commit_count {
                        state_string.push('!');
                    }

                    let upstream_opt = b.upstream.clone();
                    if let Some(upstream) = upstream_opt {
                        state_string.push('r');
                        let upstream_branch_patch: state_computation::PatchInfo = upstream
                            .patches
                            .iter()
                            .filter(|p| p.patch_id == ps_id)
                            .map(|p| p.to_owned())
                            .collect::<Vec<state_computation::PatchInfo>>()
                            .first()
                            .unwrap()
                            .clone();

                        if upstream_branch_patch.commit_diff_id != commit_diff_id {
                            state_string.push('*');
                        }

                        if upstream.patches.len() < upstream.commit_count {
                            state_string.push('!');
                        }
                    }
                    row.add_cell(None, Some(Blue), &state_string);

                    if config.list.add_extra_patch_info {
                        let hook_stdout = list::execute_list_additional_info_hook(
                            repo_root_str,
                            repo_gitdir_str,
                            &[
                                &patch.index.to_string(),
                                &state_string,
                                &patch.oid.to_string(),
                                &patch.summary,
                            ],
                        )
                        .map_err(ListError::GetHookOutputError)?;
                        let hook_stdout_len = config.list.extra_patch_info_length;
                        row.add_cell(Some(hook_stdout_len), Some(Blue), hook_stdout);
                    }
                }
                row.add_cell(Some(1), Some(Cyan), ")");
            } else {
                row.add_cell(None, Some(Cyan), "()")
            }
        } else {
            row.add_cell(None, Some(Cyan), "()")
        }

        println!("{}", row)
    }

    // TODO: handle the reorder option

    // 2 - some patch 2 (branchC)
    // 1 - some patch 1 (branchA!, branchB)
    // 3 - some patch 3 (branchD)
    // 0 - some patch 0 (branchD)

    Ok(())
}
