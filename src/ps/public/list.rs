use super::super::super::ps;
use super::super::private::config;
use super::super::private::git;
use super::super::private::git::RebaseTodoCommand;
use super::super::private::list;
use super::super::private::paths;
use super::super::private::state_computation;
use ansi_term::Color;
use ansi_term::Colour::{Blue, Cyan, Fixed, Green, Yellow};
use std::cmp::Ordering;

#[derive(Debug)]
pub enum ListError {
    RepositoryNotFound,
    GetPatchStackFailed(Box<dyn std::error::Error>),
    GetPatchListFailed(Box<dyn std::error::Error>),
    GetRepoRootPathFailed(Box<dyn std::error::Error>),
    PathNotUtf8,
    GetConfigFailed(Box<dyn std::error::Error>),
    GetCommitDiffPatchIdFailed(Box<dyn std::error::Error>),
    GetHookOutputError(Box<dyn std::error::Error>),
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
}

impl std::fmt::Display for ListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepositoryNotFound => write!(f, "repository not found"),
            Self::GetPatchStackFailed(e) => write!(f, "get patch stack failed, {}", e),
            Self::GetPatchListFailed(e) => {
                write!(f, "get patch stack list of patches failed, {}", e)
            }
            Self::GetRepoRootPathFailed(e) => write!(f, "get repository root path failed, {}", e),
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::GetConfigFailed(e) => write!(f, "get config failed, {}", e),
            Self::GetCommitDiffPatchIdFailed(e) => {
                write!(f, "get commit diff patch id failed, {}", e)
            }
            Self::GetHookOutputError(e) => write!(f, "get hook output failed, {}", e),
            Self::CurrentBranchNameMissing => write!(f, "current branch name missing"),
            Self::GetUpstreamBranchNameFailed => write!(f, "get upstream branch name failed"),
        }
    }
}

impl std::error::Error for ListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RepositoryNotFound => None,
            Self::GetPatchStackFailed(e) => Some(e.as_ref()),
            Self::GetPatchListFailed(e) => Some(e.as_ref()),
            Self::GetRepoRootPathFailed(e) => Some(e.as_ref()),
            Self::PathNotUtf8 => None,
            Self::GetConfigFailed(e) => Some(e.as_ref()),
            Self::GetCommitDiffPatchIdFailed(e) => Some(e.as_ref()),
            Self::GetHookOutputError(e) => Some(e.as_ref()),
            Self::CurrentBranchNameMissing => None,
            Self::GetUpstreamBranchNameFailed => None,
        }
    }
}

fn bg_color(
    is_connected_to_prev_row: bool,
    prev_row_showed_color: bool,
    alternate_colors: bool,
) -> Option<ansi_term::Colour> {
    let super_light_gray = Fixed(237);

    if alternate_colors {
        if (is_connected_to_prev_row && prev_row_showed_color)
            || (!is_connected_to_prev_row && !prev_row_showed_color)
        {
            Some(super_light_gray)
        } else {
            None
        }
    } else {
        None
    }
}

fn is_connected_to_prev_row(prev_patch_branches: &[String], cur_patch_branches: &[String]) -> bool {
    cur_patch_branches
        .iter()
        .map(|cb| prev_patch_branches.contains(cb))
        .reduce(|acc, v| acc || v)
        .unwrap()
}

fn rebase_todo_command_to_row(todo: &RebaseTodoCommand, color: bool) -> list::ListRow {
    let mut row = list::ListRow::new(color);
    match todo {
        RebaseTodoCommand::Pick {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Revert {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Edit {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Reword {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Squash {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Drop {
            line: _,
            key,
            sha,
            rest,
        }
        | RebaseTodoCommand::Fixup {
            line: _,
            key,
            sha,
            rest,
            keep_only_this_commits_message: _,
            open_editor: _,
        } => {
            row.add_cell(Some(11), Some(Green), None, format!("{} ", key.clone()));
            row.add_cell(Some(8), Some(Yellow), None, format!("{:.7} ", sha.clone()));
            row.add_cell(Some(51), None, None, format!("{:.50} ", rest.clone()));
            row
        }
        RebaseTodoCommand::Merge {
            line: _,
            key,
            sha,
            label,
            oneline,
            reword: _,
        } => {
            row.add_cell(Some(11), Some(Green), None, format!("{} ", key.clone()));
            row.add_cell(
                Some(8),
                Some(Yellow),
                None,
                format!("{:.7} ", sha.clone().unwrap_or("".to_string())),
            );
            row.add_cell(
                Some(51),
                None,
                None,
                format!("{:.50} ", format!("{} {}", label, oneline).to_string()),
            );
            row
        }
        RebaseTodoCommand::Exec { line: _, key, rest }
        | RebaseTodoCommand::Break { line: _, key, rest }
        | RebaseTodoCommand::Label { line: _, key, rest }
        | RebaseTodoCommand::Reset { line: _, key, rest }
        | RebaseTodoCommand::UpdateRef { line: _, key, rest }
        | RebaseTodoCommand::Noop { line: _, key, rest } => {
            row.add_cell(Some(11), Some(Green), None, format!("{} ", key.clone()));
            row.add_cell(Some(51), None, None, format!("{:.50} ", rest.clone()));
            row
        }
        RebaseTodoCommand::Comment {
            line: _,
            key,
            message,
        } => {
            row.add_cell(Some(11), Some(Green), None, format!("{} ", key.clone()));
            row.add_cell(Some(51), None, None, format!("{:.50} ", message.clone()));
            row
        }
    }
}

fn get_behind_count(
    repo: &git2::Repository,
    patch_stack: &ps::PatchStack,
    patch_stack_upstream_tracking_branch_name: &str,
) -> usize {
    let patch_stack_branch_upstream = repo
        .find_branch(
            patch_stack_upstream_tracking_branch_name,
            git2::BranchType::Remote,
        )
        .expect("cur patch stack branch upstream to exist");

    let patch_stack_branch_upstream_oid = patch_stack_branch_upstream
        .into_reference()
        .target()
        .expect("cur patch stack branch upstream to have a target");

    let behind_com_anc = git::common_ancestor(
        repo,
        patch_stack.head.target().expect("HEAD to have an oid"),
        patch_stack_branch_upstream_oid,
    )
    .expect("common ancestor between HEAD and upstream tracking branch to exist");

    git::count_commits(repo, patch_stack_branch_upstream_oid, behind_com_anc)
        .expect("to be able to count commits from remote tracking branch to common ancestor")
}

pub fn list(color: bool) -> Result<(), ListError> {
    let repo = git::create_cwd_repo().map_err(|_| ListError::RepositoryNotFound)?;

    let repo_root_path =
        paths::repo_root_path(&repo).map_err(|e| ListError::GetRepoRootPathFailed(e.into()))?;
    let repo_root_str = repo_root_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(ListError::PathNotUtf8)?;
    let config = config::get_config(repo_root_str, repo_gitdir_str)
        .map_err(|e| ListError::GetConfigFailed(e.into()))?;

    if git::in_rebase(repo_gitdir_path) {
        let rebase_head_name = git::in_rebase_head_name(repo_gitdir_path)
            .unwrap()
            .trim()
            .replace("refs/heads/", "");

        let rebase_onto = git::in_rebase_onto(repo_gitdir_path)
            .unwrap()
            .trim()
            .to_string();
        if color {
            print!(
                "{}",
                Color::Red.paint(format!(
                    "rebase of '{}' in progress; onto",
                    rebase_head_name
                ))
            );
            println!(" {}", Color::Yellow.paint(format!("{:.7} ", rebase_onto)));
        } else {
            println!(
                "rebase of '{}' in progress; onto {:.7}",
                rebase_head_name, rebase_onto
            );
        }

        if !config.list.reverse_order {
            let todos_vec = git::in_rebase_todos(repo_gitdir_path).unwrap();
            println!(
                "Next commands to do ({} remaining commands)",
                todos_vec.len()
            );
            for todo in todos_vec.iter().rev() {
                println!("{}", rebase_todo_command_to_row(todo, color));
            }
            println!("(use \"git rebase --edit-todo\" to view and edit)");
            println!("(use \"git rebase --continue\" once you are satisfied with your changes)");
            println!();
        }
    }

    let cur_patch_stack_branch_ref = match git::in_rebase(repo_gitdir_path) {
        true => git::in_rebase_head_name(repo_gitdir_path)
            .unwrap()
            .trim()
            .to_string(),
        false => git::get_current_branch(&repo).ok_or(ListError::CurrentBranchNameMissing)?,
    };
    let cur_patch_stack_branch_upstream_ref =
        git::branch_upstream_name(&repo, &cur_patch_stack_branch_ref)
            .map_err(|_| ListError::GetUpstreamBranchNameFailed)?;
    let cur_patch_stack_branch_name = str::replace(&cur_patch_stack_branch_ref, "refs/heads/", "");
    let cur_patch_stack_branch_upstream_name =
        str::replace(&cur_patch_stack_branch_upstream_ref, "refs/remotes/", "");

    // We do know what branch we are currently checked out on when running this command. It seems
    // like we should use that as the base branch.

    let patch_stack =
        ps::get_patch_stack(&repo).map_err(|e| ListError::GetPatchStackFailed(e.into()))?;

    let list_of_patches = ps::get_patch_list(&repo, &patch_stack)
        .map_err(|e| ListError::GetPatchListFailed(e.into()))?;

    let base_oid = patch_stack.base.target().unwrap();

    let patch_info_collection =
        state_computation::get_list_patch_info(&repo, base_oid, &cur_patch_stack_branch_name)
            .unwrap();

    let behind_count = get_behind_count(&repo, &patch_stack, &cur_patch_stack_branch_upstream_name);

    println!(
        "{} tracking {} [ahead {}, behind {}]",
        &cur_patch_stack_branch_name,
        &cur_patch_stack_branch_upstream_name,
        list_of_patches.len(),
        behind_count,
    );

    let list_of_patches_iter: Box<dyn Iterator<Item = _>> = if config.list.reverse_order {
        Box::new(list_of_patches.into_iter())
    } else {
        Box::new(list_of_patches.into_iter().rev())
    };

    let mut prev_patch_branches: Vec<String> = vec![];
    let mut connected_to_prev_row: bool;
    let mut prev_row_included_bg: bool = true;

    for patch in list_of_patches_iter {
        let mut row = list::ListRow::new(color);

        let commit = repo.find_commit(patch.oid).unwrap();

        let commit_diff_id: Option<git2::Oid> = match git::commit_diff_patch_id(&repo, &commit) {
            Ok(id) => Some(id),
            Err(git::CommitDiffPatchIdError::GetDiffFailed(git::CommitDiffError::MergeCommit)) => {
                None
            }
            Err(e) => return Err(ListError::GetCommitDiffPatchIdFailed(e.into())),
        };

        if let Some(ps_id) = ps::commit_ps_id(&commit) {
            if let Some(patch_info) = patch_info_collection.get(&ps_id) {
                let cur_row_branches: Vec<String> =
                    patch_info.branches.iter().map(|b| b.name.clone()).collect();
                connected_to_prev_row =
                    is_connected_to_prev_row(&prev_patch_branches, &cur_row_branches);
                prev_patch_branches = cur_row_branches.to_vec();
            } else {
                connected_to_prev_row = false;
                prev_patch_branches = vec![];
            }
        } else {
            connected_to_prev_row = false;
            prev_patch_branches = vec![];
        }

        let bg_color = bg_color(connected_to_prev_row, prev_row_included_bg, config.list.alternate_colors);
        prev_row_included_bg = bg_color.is_some();

        row.add_cell(Some(5), Some(Green), bg_color, format!("{} ", patch.index));
        row.add_cell(
            Some(8),
            Some(Yellow),
            bg_color,
            format!("{:.7} ", patch.oid),
        );
        row.add_cell(
            Some(51),
            None,
            bg_color,
            format!("{:.50} ", patch.summary.clone()),
        );

        if let Some(ps_id) = ps::commit_ps_id(&commit) {
            if let Some(patch_info) = patch_info_collection.get(&ps_id) {
                row.add_cell(Some(2), Some(Cyan), bg_color, "( ");
                for b in patch_info.branches.iter() {
                    match patch_info.branches.len().cmp(&1) {
                        Ordering::Greater => {
                            row.add_cell(None, None, bg_color, format!("{} ", b.name.clone()));
                        }
                        Ordering::Less => {}
                        Ordering::Equal => {
                            let branch_info = patch_info.branches.first().unwrap();
                            if !branch_info.name.starts_with("ps/rr/") {
                                row.add_cell(None, None, bg_color, format!("{} ", b.name.clone()));
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

                    match commit_diff_id {
                        Some(id) => {
                            if branch_patch.commit_diff_id != id {
                                state_string.push('*');
                            }
                        }
                        None => state_string.push('*'),
                    }

                    let upstream_opt = b.upstream.clone();
                    if let Some(upstream) = upstream_opt {
                        state_string.push('r');
                        let upstream_branch_patch_opt: Option<state_computation::PatchInfo> =
                            upstream
                                .patches
                                .iter()
                                .filter(|p| p.patch_id == ps_id)
                                .map(|p| p.to_owned())
                                .collect::<Vec<state_computation::PatchInfo>>()
                                .first()
                                .cloned();

                        match commit_diff_id {
                            Some(id) => {
                                if let Some(upstream_branch_patch) = upstream_branch_patch_opt {
                                    if upstream_branch_patch.commit_diff_id != id {
                                        state_string.push('*');
                                    }
                                }
                            }
                            None => state_string.push('*'),
                        }

                        if upstream.patches.len() < upstream.commit_count {
                            state_string.push('!');
                        }
                    }
                    row.add_cell(None, Some(Blue), bg_color, format!("{} ", &state_string));

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
                        .map_err(|e| ListError::GetHookOutputError(e.into()))?;
                        let hook_stdout_len = config.list.extra_patch_info_length;
                        row.add_cell(
                            Some(hook_stdout_len + 1),
                            Some(Blue),
                            bg_color,
                            format!("{} ", hook_stdout),
                        );
                    }
                }
                row.add_cell(Some(2), Some(Cyan), bg_color, ")");
            } else {
                row.add_cell(None, Some(Cyan), bg_color, "()")
            }
        } else {
            row.add_cell(None, Some(Cyan), bg_color, "()")
        }

        println!("{}", row);
    }

    if git::in_rebase(repo_gitdir_path) && config.list.reverse_order {
        let todos_vec = git::in_rebase_todos(repo_gitdir_path).unwrap();
        println!();
        println!(
            "Next commands to do ({} remaining commands)",
            todos_vec.len()
        );
        for todo in todos_vec.iter() {
            println!("{}", rebase_todo_command_to_row(todo, color));
        }
        println!("(use \"git rebase --edit-todo\" to view and edit)");
        println!("(use \"git rebase --continue\" once you are satisfied with your changes)");
        println!();
    }

    Ok(())
}
