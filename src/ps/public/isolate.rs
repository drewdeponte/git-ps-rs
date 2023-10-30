use super::super::super::ps;
use super::super::private::cherry_picking;
use super::super::private::git;
use super::super::private::hooks;
use super::super::private::paths;
use super::super::private::string_file_io::{read_str_from_file, write_str_to_file};
use super::super::private::utils;
use std::result::Result;

#[derive(Debug)]
pub enum IsolateError {
    OpenGitRepositoryFailed(Box<dyn std::error::Error>),
    OpenGitConfigFailed(Box<dyn std::error::Error>),
    UncommittedChangesExistFailure(Box<dyn std::error::Error>),
    UncommittedChangesExist,
    GetPatchStackFailed(Box<dyn std::error::Error>),
    GetPatchListFailed(Box<dyn std::error::Error>),
    PatchIndexNotFound,
    PatchStackBaseNotFound,
    CreateBranchFailed,
    BranchNameNotUtf8,
    MergeCommitDetected(String),
    ConflictsExist(String, String),
    FailedToCheckout(Box<dyn std::error::Error>),
    GetCurrentBranchFailed,
    StoreLastBranchFailed(Box<dyn std::error::Error>),
    ReadLastBranchFailed(Box<dyn std::error::Error>),
    GetRepoRootPathFailed(Box<dyn std::error::Error>),
    PathNotUtf8,
    HookNotFound(Box<dyn std::error::Error>),
    HookExecutionFailed(Box<dyn std::error::Error>),
    FindIsolateBranchFailed(Box<dyn std::error::Error>),
    DeleteIsolateBranchFailed(Box<dyn std::error::Error>),
    FailedToMapIndexesForCherryPick(Box<dyn std::error::Error>),
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetRemoteNameFailed,
    RemoteNameNotUtf8,
    FindRemoteFailed(Box<dyn std::error::Error>),
    RemoteUrlNotUtf8,
    Unhandled(Box<dyn std::error::Error>),
}

impl From<cherry_picking::CherryPickError> for IsolateError {
    fn from(value: cherry_picking::CherryPickError) -> Self {
        match value {
            cherry_picking::CherryPickError::MergeCommitDetected(oid_string) => {
                Self::MergeCommitDetected(oid_string)
            }
            cherry_picking::CherryPickError::ConflictsExist(oid_a_string, oid_b_string) => {
                Self::ConflictsExist(oid_a_string, oid_b_string)
            }
            _ => Self::Unhandled(value.into()),
        }
    }
}

impl std::fmt::Display for IsolateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenGitRepositoryFailed(e) => write!(f, "failed to open git repository, {}", e),
            Self::OpenGitConfigFailed(e) => write!(f, "failed to open git config, {}", e),
            Self::UncommittedChangesExistFailure(e) => {
                write!(f, "checking for uncommitted changes failed, {}", e)
            }
            Self::UncommittedChangesExist => write!(f, "uncommited changes exist"),
            Self::GetPatchStackFailed(e) => write!(f, "get patch stack failed, {}", e),
            Self::GetPatchListFailed(e) => {
                write!(f, "get patch stack list of patches failed, {}", e)
            }
            Self::PatchIndexNotFound => write!(f, "patch index not found"),
            Self::PatchStackBaseNotFound => write!(f, "patch stack base not found"),
            Self::CreateBranchFailed => write!(f, "create branch failed"),
            Self::BranchNameNotUtf8 => write!(f, "branch name not utf-8"),
            Self::MergeCommitDetected(oid) => {
                write!(f, "merge commit detected with sha {}", oid)
            }
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict(s) detected when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::FailedToCheckout(e) => write!(f, "failed to checkout branch, {}", e),
            Self::GetCurrentBranchFailed => write!(f, "failed to get current branch"),
            Self::StoreLastBranchFailed(e) => write!(f, "failed to store last branch, {}", e),
            Self::ReadLastBranchFailed(e) => write!(f, "failed to read last branch, {}", e),
            Self::GetRepoRootPathFailed(e) => {
                write!(f, "failed to get repositories root path, {}", e)
            }
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::HookNotFound(e) => write!(f, "hook not found, {}", e),
            Self::HookExecutionFailed(e) => write!(f, "hook execution failed, {}", e),
            Self::FindIsolateBranchFailed(e) => write!(f, "failed to find isolate branch, {}", e),
            Self::DeleteIsolateBranchFailed(e) => {
                write!(f, "failed to delete isolate branch, {}", e)
            }
            Self::FailedToMapIndexesForCherryPick(e) => {
                write!(f, "failed to map indexes for cherry pick, {}", e)
            }
            Self::CurrentBranchNameMissing => write!(f, "current branch name missing"),
            Self::GetUpstreamBranchNameFailed => write!(f, "failed to get upstream branch name"),
            Self::GetRemoteNameFailed => write!(f, "fail to get remote name"),
            Self::RemoteNameNotUtf8 => write!(f, "remote name not utf-8"),
            Self::FindRemoteFailed(e) => write!(f, "failed to find remote, {}", e),
            Self::RemoteUrlNotUtf8 => write!(f, "remote url not utf-8"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for IsolateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenGitRepositoryFailed(e) => Some(e.as_ref()),
            Self::OpenGitConfigFailed(e) => Some(e.as_ref()),
            Self::UncommittedChangesExistFailure(e) => Some(e.as_ref()),
            Self::UncommittedChangesExist => None,
            Self::GetPatchStackFailed(e) => Some(e.as_ref()),
            Self::GetPatchListFailed(e) => Some(e.as_ref()),
            Self::PatchIndexNotFound => None,
            Self::PatchStackBaseNotFound => None,
            Self::CreateBranchFailed => None,
            Self::BranchNameNotUtf8 => None,
            Self::MergeCommitDetected(_) => None,
            Self::ConflictsExist(_, _) => None,
            Self::FailedToCheckout(e) => Some(e.as_ref()),
            Self::GetCurrentBranchFailed => None,
            Self::StoreLastBranchFailed(e) => Some(e.as_ref()),
            Self::ReadLastBranchFailed(e) => Some(e.as_ref()),
            Self::GetRepoRootPathFailed(e) => Some(e.as_ref()),
            Self::PathNotUtf8 => None,
            Self::HookNotFound(e) => Some(e.as_ref()),
            Self::HookExecutionFailed(e) => Some(e.as_ref()),
            Self::FindIsolateBranchFailed(e) => Some(e.as_ref()),
            Self::DeleteIsolateBranchFailed(e) => Some(e.as_ref()),
            Self::FailedToMapIndexesForCherryPick(e) => Some(e.as_ref()),
            Self::CurrentBranchNameMissing => None,
            Self::GetUpstreamBranchNameFailed => None,
            Self::GetRemoteNameFailed => None,
            Self::RemoteNameNotUtf8 => None,
            Self::FindRemoteFailed(e) => Some(e.as_ref()),
            Self::RemoteUrlNotUtf8 => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn isolate(
    start_patch_index_optional: Option<usize>,
    end_patch_index_optional: Option<usize>,
    color: bool,
) -> Result<(), IsolateError> {
    let isolate_branch_name = "ps/tmp/isolate";
    let repo = ps::private::git::create_cwd_repo()
        .map_err(|e| IsolateError::OpenGitRepositoryFailed(e.into()))?;

    let repo_gitdir_path = repo.path();
    let repo_gitdir_str = repo_gitdir_path.to_str().ok_or(IsolateError::PathNotUtf8)?;
    let config =
        git2::Config::open_default().map_err(|e| IsolateError::OpenGitConfigFailed(e.into()))?;

    if git::uncommitted_changes_exist(&repo)
        .map_err(|e| IsolateError::UncommittedChangesExistFailure(e.into()))?
    {
        return Err(IsolateError::UncommittedChangesExist);
    }

    match start_patch_index_optional {
        Some(patch_index) => {
            let patch_stack = ps::get_patch_stack(&repo)
                .map_err(|e| IsolateError::GetPatchStackFailed(e.into()))?;
            let patches_vec = ps::get_patch_list(&repo, &patch_stack)
                .map_err(|e| IsolateError::GetPatchListFailed(e.into()))?;
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
            .map_err(|e| IsolateError::FailedToMapIndexesForCherryPick(e.into()))?;

            cherry_picking::cherry_pick(
                &repo,
                &config,
                cherry_pick_range.root_oid,
                cherry_pick_range.leaf_oid,
                branch_ref_name,
                0,
                false,
                true,
            )?;

            // get currently checked out branch name
            let checked_out_branch = git::get_current_branch_shorthand(&repo)
                .ok_or(IsolateError::GetCurrentBranchFailed)?;
            // write currently checked out branch name to disk
            let path = paths::isolate_last_branch_path(&repo);
            write_str_to_file(checked_out_branch.as_str(), path)
                .map_err(|e| IsolateError::StoreLastBranchFailed(e.into()))?;

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
                .map_err(|e| IsolateError::FindRemoteFailed(e.into()))?;
            let remote_url_str = remote.url().ok_or(IsolateError::RemoteUrlNotUtf8)?;

            // checkout the ps/tmp/checkout branch
            utils::execute("git", &["checkout", isolate_branch_name])
                .map_err(|e| IsolateError::FailedToCheckout(e.into()))?;

            let repo_root_path = paths::repo_root_path(&repo)
                .map_err(|e| IsolateError::GetRepoRootPathFailed(e.into()))?;
            let repo_root_str = repo_root_path.to_str().ok_or(IsolateError::PathNotUtf8)?;
            match hooks::find_hook(repo_root_str, repo_gitdir_str, "isolate_post_checkout") {
                Ok(hook_path) => utils::execute(
                    hook_path.to_str().ok_or(IsolateError::PathNotUtf8)?,
                    &[remote_name_str, remote_url_str],
                )
                .map_err(|e| IsolateError::HookExecutionFailed(e.into()))?,
                Err(hooks::FindHookError::NotFound) => {}
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
                Err(e) => return Err(IsolateError::HookNotFound(e.into())),
            }

            Ok(())
        }
        None => {
            // read last checked out branch name from disk
            let path = paths::isolate_last_branch_path(&repo);
            let last_branch = read_str_from_file(path)
                .map_err(|e| IsolateError::ReadLastBranchFailed(e.into()))?;

            // check it out
            utils::execute("git", &["checkout", &last_branch])
                .map_err(|e| IsolateError::FailedToCheckout(e.into()))?;

            let mut isolate_branch = repo
                .find_branch(isolate_branch_name, git2::BranchType::Local)
                .map_err(|e| IsolateError::FindIsolateBranchFailed(e.into()))?;
            isolate_branch
                .delete()
                .map_err(|e| IsolateError::DeleteIsolateBranchFailed(e.into()))?;

            let repo_root_path = paths::repo_root_path(&repo)
                .map_err(|e| IsolateError::GetRepoRootPathFailed(e.into()))?;
            let repo_root_str = repo_root_path.to_str().ok_or(IsolateError::PathNotUtf8)?;

            match hooks::find_hook(repo_root_str, repo_gitdir_str, "isolate_post_cleanup") {
                Ok(hook_path) => {
                    utils::execute(hook_path.to_str().ok_or(IsolateError::PathNotUtf8)?, &[])
                        .map_err(|e| IsolateError::HookExecutionFailed(e.into()))?
                }
                Err(hooks::FindHookError::NotFound) => {}
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
                Err(e) => return Err(IsolateError::HookNotFound(e.into())),
            }

            Ok(())
        }
    }
}
