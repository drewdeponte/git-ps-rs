use crate::ps::commit_ps_id;

use super::super::super::ps;
use super::super::private::git;
use super::super::private::state_computation;
use super::paths;
use std::collections::HashMap;
use std::fmt;
use std::result::Result;
use uuid::Uuid;

#[derive(Debug)]
pub enum RequestReviewBranchError {
    RepositoryMissing,
    PatchStackNotFound,
    PatchStackBaseNotFound,
    PatchIndexNotFound,
    PatchCommitNotFound,
    PatchMessageMissing,
    AddPsIdToPatchFailed(ps::AddPsIdError),
    PatchSummaryMissing,
    CreateRrBranchFailed,
    RrBranchNameNotUtf8,
    CherryPickFailed(git::GitError),
    GetPatchListFailed(ps::GetPatchListError),
    GetPatchMetaDataPathFailed(paths::PathsError),
    OpenGitConfigFailed(git2::Error),
    PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
    PatchStackHeadNoName,
    GetListPatchInfoFailed(state_computation::GetListPatchInfoError),
    PatchBranchAmbiguous,
    AddPatchIdsFailed(ps::AddPatchIdsError),
    PatchIndexOutOfStackRange(usize),
    AssociatedBranchAmbiguous(std::vec::Vec<String>),
    PatchSeriesRequireBranchName,
}

impl From<git::CreateCwdRepositoryError> for RequestReviewBranchError {
    fn from(_e: git::CreateCwdRepositoryError) -> Self {
        RequestReviewBranchError::RepositoryMissing
    }
}

impl From<ps::PatchStackError> for RequestReviewBranchError {
    fn from(e: ps::PatchStackError) -> Self {
        match e {
            ps::PatchStackError::GitError(_git2_error) => {
                RequestReviewBranchError::PatchStackNotFound
            }
            ps::PatchStackError::HeadNoName => RequestReviewBranchError::PatchStackNotFound,
            ps::PatchStackError::UpstreamBranchNameNotFound => {
                RequestReviewBranchError::PatchStackNotFound
            }
        }
    }
}

impl From<ps::AddPsIdError> for RequestReviewBranchError {
    fn from(e: ps::AddPsIdError) -> Self {
        RequestReviewBranchError::AddPsIdToPatchFailed(e)
    }
}

impl fmt::Display for RequestReviewBranchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestReviewBranchError::RepositoryMissing => {
                write!(f, "Repository not found in current working directory")
            }
            RequestReviewBranchError::PatchStackNotFound => write!(f, "Patch Stack not found"),
            RequestReviewBranchError::PatchStackBaseNotFound => {
                write!(f, "Patch Stack Base not found")
            }
            RequestReviewBranchError::PatchIndexNotFound => write!(f, "Patch Index out of range"),
            RequestReviewBranchError::PatchCommitNotFound => write!(f, "Patch commit not found"),
            RequestReviewBranchError::PatchMessageMissing => write!(f, "Patch missing message"),
            RequestReviewBranchError::AddPsIdToPatchFailed(_add_ps_id_error) => {
                write!(f, "Failed to add patch stack id to patch")
            }
            RequestReviewBranchError::PatchSummaryMissing => write!(f, "Patch missing summary"),
            RequestReviewBranchError::CreateRrBranchFailed => {
                write!(f, "Failed to create request-review branch")
            }
            RequestReviewBranchError::RrBranchNameNotUtf8 => {
                write!(f, "request-review branch is not utf8")
            }
            RequestReviewBranchError::CherryPickFailed(_git_error) => {
                write!(f, "Failed to cherry pick")
            }
            RequestReviewBranchError::GetPatchListFailed(_patch_list_error) => {
                write!(f, "Failed to get patch list")
            }
            RequestReviewBranchError::GetPatchMetaDataPathFailed(_patch_meta_data_path_error) => {
                write!(
                    f,
                    "Failed to get patch meta data path {:?}",
                    _patch_meta_data_path_error
                )
            }
            RequestReviewBranchError::OpenGitConfigFailed(_) => {
                write!(f, "Failed to open git config")
            }
            RequestReviewBranchError::PatchCommitDiffPatchIdFailed(_) => {
                write!(f, "Failed to get commit diff patch id")
            }
            RequestReviewBranchError::PatchStackHeadNoName => {
                write!(f, "Patch Stack Head has no name")
            }
            RequestReviewBranchError::GetListPatchInfoFailed(_get_list_patch_info_error) => {
                write!(f, "Failed to get list of patch Git info")
            }
            RequestReviewBranchError::PatchBranchAmbiguous => {
                write!(
                    f,
                    "Patch Branch is Ambiguous - more than one branch associated with patch"
                )
            }
            RequestReviewBranchError::AddPatchIdsFailed(_) => {
                write!(f, "Failed to add patch ids to commits in the patch stack")
            }
            RequestReviewBranchError::PatchIndexOutOfStackRange(_) => {
                write!(f, "Patch index out of patch stack range")
            }
            RequestReviewBranchError::AssociatedBranchAmbiguous(_) => {
                write!(
                    f,
                    "The associated branch is ambiguous. Please specify the branch explicitly."
                )
            }
            RequestReviewBranchError::PatchSeriesRequireBranchName => {
                write!(
                    f,
                    "When creating a patch series you must specify the branch name."
                )
            }
        }
    }
}

pub fn request_review_branch(
    repo: &git2::Repository,
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    given_branch_name_option: Option<String>,
) -> Result<(git2::Branch<'_>, git2::Oid), RequestReviewBranchError> {
    let config =
        git2::Config::open_default().map_err(RequestReviewBranchError::OpenGitConfigFailed)?;

    ps::add_patch_ids(repo, &config).map_err(RequestReviewBranchError::AddPatchIdsFailed)?;

    let patch_stack = ps::get_patch_stack(repo)?;
    let patches_vec = ps::get_patch_list(repo, &patch_stack)
        .map_err(RequestReviewBranchError::GetPatchListFailed)?;

    // validate patch indexes are within bounds
    if start_patch_index > (patches_vec.len() - 1) {
        return Err(RequestReviewBranchError::PatchIndexOutOfStackRange(
            start_patch_index,
        ));
    }

    if let Some(end_index) = end_patch_index {
        if end_index > (patches_vec.len() - 1) {
            return Err(RequestReviewBranchError::PatchIndexOutOfStackRange(
                end_index,
            ));
        }
    }

    // fetch computed state from Git tree
    let patch_stack_base_commit = patch_stack
        .base
        .peel_to_commit()
        .map_err(|_| RequestReviewBranchError::PatchStackBaseNotFound)?;

    let head_ref_name = patch_stack
        .head
        .shorthand()
        .ok_or(RequestReviewBranchError::PatchStackHeadNoName)?;

    let patch_info_collection: HashMap<Uuid, state_computation::PatchGitInfo> =
        state_computation::get_list_patch_info(repo, patch_stack_base_commit.id(), head_ref_name)
            .map_err(RequestReviewBranchError::GetListPatchInfoFailed)?;

    // collect vector of indexes
    let indexes_iter = match end_patch_index {
        Some(end_index) => start_patch_index..=end_index,
        None => start_patch_index..=start_patch_index,
    };

    // get unique branch names of patches in series
    let mut range_patch_branches: Vec<String> = indexes_iter
        .clone()
        .map(|i| patches_vec.get(i).unwrap())
        .map(|lp| {
            let commit = repo.find_commit(lp.oid).unwrap();
            commit_ps_id(&commit).unwrap()
        })
        .filter_map(|id| patch_info_collection.get(&id))
        .flat_map(|pi| pi.branches.iter().map(|b| b.name.clone()))
        .collect();
    range_patch_branches.sort();
    range_patch_branches.dedup();

    // figure out the new branch name, either generate a new one, use the associated one, or
    // require user to explicitly specify
    let new_branch_name: String;
    if let Some(given_branch_name) = given_branch_name_option {
        new_branch_name = given_branch_name;
    } else if range_patch_branches.is_empty() {
        if end_patch_index.is_none() {
            let patch_oid = patches_vec.get(indexes_iter.last().unwrap()).unwrap().oid;
            let patch_commit = repo.find_commit(patch_oid).unwrap();
            let patch_summary = patch_commit.summary().expect("Patch Missing Summary");
            new_branch_name = ps::generate_rr_branch_name(patch_summary);
        } else {
            return Err(RequestReviewBranchError::PatchSeriesRequireBranchName);
        }
    } else if range_patch_branches.len() == 1 {
        new_branch_name = range_patch_branches.first().unwrap().to_string()
    } else {
        return Err(RequestReviewBranchError::AssociatedBranchAmbiguous(
            range_patch_branches.clone(),
        ));
    }

    // create branch on top of the patch stack base
    let branch = repo
        .branch(new_branch_name.as_str(), &patch_stack_base_commit, true)
        .map_err(|_| RequestReviewBranchError::CreateRrBranchFailed)?;

    let branch_ref_name = branch
        .get()
        .name()
        .ok_or(RequestReviewBranchError::RrBranchNameNotUtf8)?;

    let start_patch_oid = patches_vec.get(start_patch_index).unwrap().oid;
    let start_patch_commit = repo.find_commit(start_patch_oid).unwrap();
    let start_patch_parent_commit = start_patch_commit.parent(0).unwrap();
    let start_patch_parent_oid = start_patch_parent_commit.id();

    let last_commit_oid_cherry_picked = match end_patch_index {
        Some(end_index) => {
            let end_patch_oid = patches_vec.get(end_index).unwrap().oid;
            ps::cherry_pick_no_working_copy_range(
                repo,
                &config,
                start_patch_parent_oid,
                end_patch_oid,
                branch_ref_name,
                1,
                false,
            )
        }
        None => ps::cherry_pick_no_working_copy_range(
            repo,
            &config,
            start_patch_parent_oid,
            start_patch_oid,
            branch_ref_name,
            1,
            false,
        ),
    }
    .map_err(RequestReviewBranchError::CherryPickFailed)?
    .expect("No commits cherry picked, when we expected at least one");

    Ok((branch, last_commit_oid_cherry_picked))
}
