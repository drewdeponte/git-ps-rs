use super::super::super::ps;
use super::super::private::git;
use super::super::private::state_computation;
use super::paths;
use std::collections::HashMap;
use std::fmt;
use std::result::Result;
use uuid::Uuid;

#[derive(Debug)]
pub enum BranchError {
    RepositoryMissing,
    PatchStackNotFound,
    PatchStackBaseNotFound,
    PatchIndexNotFound,
    PatchCommitNotFound,
    PatchMessageMissing,
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
    AssociatedBranchAmbiguous(std::vec::Vec<String>),
    PatchSeriesRequireBranchName,
    PatchIndexRangeOutOfBounds(ps::PatchRangeWithinStackBoundsError),
}

impl From<git::CreateCwdRepositoryError> for BranchError {
    fn from(_e: git::CreateCwdRepositoryError) -> Self {
        BranchError::RepositoryMissing
    }
}

impl From<ps::PatchStackError> for BranchError {
    fn from(e: ps::PatchStackError) -> Self {
        match e {
            ps::PatchStackError::GitError(_git2_error) => {
                BranchError::PatchStackNotFound
            }
            ps::PatchStackError::HeadNoName => BranchError::PatchStackNotFound,
            ps::PatchStackError::UpstreamBranchNameNotFound => {
                BranchError::PatchStackNotFound
            }
        }
    }
}

impl fmt::Display for BranchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BranchError::RepositoryMissing => {
                write!(f, "Repository not found in current working directory")
            }
            BranchError::PatchStackNotFound => write!(f, "Patch Stack not found"),
            BranchError::PatchStackBaseNotFound => {
                write!(f, "Patch Stack Base not found")
            }
            BranchError::PatchIndexNotFound => write!(f, "Patch Index out of range"),
            BranchError::PatchCommitNotFound => write!(f, "Patch commit not found"),
            BranchError::PatchMessageMissing => write!(f, "Patch missing message"),
            BranchError::PatchSummaryMissing => write!(f, "Patch missing summary"),
            BranchError::CreateRrBranchFailed => {
                write!(f, "Failed to create request-review branch")
            }
            BranchError::RrBranchNameNotUtf8 => {
                write!(f, "request-review branch is not utf8")
            }
            BranchError::CherryPickFailed(_git_error) => {
                write!(f, "Failed to cherry pick")
            }
            BranchError::GetPatchListFailed(_patch_list_error) => {
                write!(f, "Failed to get patch list")
            }
            BranchError::GetPatchMetaDataPathFailed(_patch_meta_data_path_error) => {
                write!(
                    f,
                    "Failed to get patch meta data path {:?}",
                    _patch_meta_data_path_error
                )
            }
            BranchError::OpenGitConfigFailed(_) => {
                write!(f, "Failed to open git config")
            }
            BranchError::PatchCommitDiffPatchIdFailed(_) => {
                write!(f, "Failed to get commit diff patch id")
            }
            BranchError::PatchStackHeadNoName => {
                write!(f, "Patch Stack Head has no name")
            }
            BranchError::GetListPatchInfoFailed(_get_list_patch_info_error) => {
                write!(f, "Failed to get list of patch Git info")
            }
            BranchError::PatchBranchAmbiguous => {
                write!(
                    f,
                    "Patch Branch is Ambiguous - more than one branch associated with patch"
                )
            }
            BranchError::AddPatchIdsFailed(_) => {
                write!(f, "Failed to add patch ids to commits in the patch stack")
            }
            BranchError::PatchIndexRangeOutOfBounds(_) => {
                write!(f, "Patch index range out of patch stack bounds")
            }
            BranchError::AssociatedBranchAmbiguous(_) => {
                write!(
                    f,
                    "The associated branch is ambiguous. Please specify the branch explicitly."
                )
            }
            BranchError::PatchSeriesRequireBranchName => {
                write!(
                    f,
                    "When creating a patch series you must specify the branch name."
                )
            }
        }
    }
}

pub fn branch(
    repo: &git2::Repository,
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    given_branch_name_option: Option<String>,
) -> Result<(git2::Branch<'_>, git2::Oid), BranchError> {
    let config =
        git2::Config::open_default().map_err(BranchError::OpenGitConfigFailed)?;

    ps::add_patch_ids(repo, &config).map_err(BranchError::AddPatchIdsFailed)?;

    let patch_stack = ps::get_patch_stack(repo)?;
    let patches_vec = ps::get_patch_list(repo, &patch_stack)
        .map_err(BranchError::GetPatchListFailed)?;

    // validate patch indexes are within bounds
    ps::patch_range_within_stack_bounds(start_patch_index, end_patch_index, &patches_vec)
        .map_err(BranchError::PatchIndexRangeOutOfBounds)?;

    // fetch computed state from Git tree
    let patch_stack_base_commit = patch_stack
        .base
        .peel_to_commit()
        .map_err(|_| BranchError::PatchStackBaseNotFound)?;

    let head_ref_name = patch_stack
        .head
        .shorthand()
        .ok_or(BranchError::PatchStackHeadNoName)?;

    let patch_info_collection: HashMap<Uuid, state_computation::PatchGitInfo> =
        state_computation::get_list_patch_info(repo, patch_stack_base_commit.id(), head_ref_name)
            .map_err(BranchError::GetListPatchInfoFailed)?;

    // collect vector of indexes
    let indexes_iter = match end_patch_index {
        Some(end_index) => start_patch_index..=end_index,
        None => start_patch_index..=start_patch_index,
    };

    // get unique branch names of patches in series
    let range_patch_branches = ps::patch_series_unique_branch_names(
        repo,
        &patches_vec,
        &patch_info_collection,
        start_patch_index,
        end_patch_index,
    );

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
            return Err(BranchError::PatchSeriesRequireBranchName);
        }
    } else if range_patch_branches.len() == 1 {
        new_branch_name = range_patch_branches.first().unwrap().to_string()
    } else {
        return Err(BranchError::AssociatedBranchAmbiguous(
            range_patch_branches.clone(),
        ));
    }

    // create branch on top of the patch stack base
    let branch = repo
        .branch(new_branch_name.as_str(), &patch_stack_base_commit, true)
        .map_err(|_| BranchError::CreateRrBranchFailed)?;

    let branch_ref_name = branch
        .get()
        .name()
        .ok_or(BranchError::RrBranchNameNotUtf8)?;

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
    .map_err(BranchError::CherryPickFailed)?
    .expect("No commits cherry picked, when we expected at least one");

    Ok((branch, last_commit_oid_cherry_picked))
}
