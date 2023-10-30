use super::super::super::ps;
use super::super::private::cherry_picking;
use super::super::private::git;
use super::super::private::state_computation;
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
    MergeCommitDetected(String),
    ConflictsExist(String, String),
    GetPatchListFailed(Box<dyn std::error::Error>),
    OpenGitConfigFailed(Box<dyn std::error::Error>),
    PatchStackHeadNoName,
    GetListPatchInfoFailed(Box<dyn std::error::Error>),
    PatchBranchAmbiguous,
    AddPatchIdsFailed(Box<dyn std::error::Error>),
    AssociatedBranchAmbiguous(std::vec::Vec<String>),
    PatchSeriesRequireBranchName,
    PatchIndexRangeOutOfBounds(Box<dyn std::error::Error>),
    UnhandledError(Box<dyn std::error::Error>),
}

impl From<git::CreateCwdRepositoryError> for BranchError {
    fn from(_e: git::CreateCwdRepositoryError) -> Self {
        BranchError::RepositoryMissing
    }
}

impl From<ps::PatchStackError> for BranchError {
    fn from(e: ps::PatchStackError) -> Self {
        match e {
            ps::PatchStackError::GitError(_git2_error) => BranchError::PatchStackNotFound,
            ps::PatchStackError::HeadNoName => BranchError::PatchStackNotFound,
            ps::PatchStackError::UpstreamBranchNameNotFound => BranchError::PatchStackNotFound,
        }
    }
}

impl From<cherry_picking::CherryPickError> for BranchError {
    fn from(value: cherry_picking::CherryPickError) -> Self {
        match value {
            cherry_picking::CherryPickError::MergeCommitDetected(oid) => {
                Self::MergeCommitDetected(oid)
            }
            cherry_picking::CherryPickError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            _ => Self::UnhandledError(value.into()),
        }
    }
}

impl From<ps::AddPatchIdsError> for BranchError {
    fn from(value: ps::AddPatchIdsError) -> Self {
        match value {
            ps::AddPatchIdsError::MergeCommitDetected(oid) => Self::MergeCommitDetected(oid),
            ps::AddPatchIdsError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            _ => Self::AddPatchIdsFailed(value.into()),
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
            BranchError::MergeCommitDetected(oid) => {
                write!(f, "merge commit detected with sha {}", oid)
            }
            BranchError::ConflictsExist(src_oid, dst_oid) => {
                write!(
                    f,
                    "conflicts exist when attempting to play commit ({}) onto commit ({})",
                    src_oid, dst_oid
                )
            }
            BranchError::GetPatchListFailed(_patch_list_error) => {
                write!(f, "Failed to get patch list")
            }
            BranchError::OpenGitConfigFailed(_) => {
                write!(f, "Failed to open git config")
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
            BranchError::UnhandledError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BranchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RepositoryMissing
            | Self::PatchStackNotFound
            | Self::PatchStackBaseNotFound
            | Self::PatchIndexNotFound
            | Self::PatchCommitNotFound
            | Self::PatchMessageMissing
            | Self::PatchSummaryMissing
            | Self::CreateRrBranchFailed
            | Self::RrBranchNameNotUtf8
            | Self::MergeCommitDetected(_)
            | Self::ConflictsExist(_, _) => None,
            Self::GetPatchListFailed(e) => Some(e.as_ref()),
            Self::OpenGitConfigFailed(e) => Some(e.as_ref()),
            Self::PatchStackHeadNoName => None,
            Self::GetListPatchInfoFailed(e) => Some(e.as_ref()),
            Self::PatchBranchAmbiguous => None,
            Self::AddPatchIdsFailed(e) => Some(e.as_ref()),
            Self::AssociatedBranchAmbiguous(_) => None,
            Self::PatchSeriesRequireBranchName => None,
            Self::PatchIndexRangeOutOfBounds(e) => Some(e.as_ref()),
            Self::UnhandledError(e) => Some(e.as_ref()),
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
        git2::Config::open_default().map_err(|e| BranchError::OpenGitConfigFailed(e.into()))?;

    ps::add_patch_ids(repo, &config)?;

    let patch_stack = ps::get_patch_stack(repo)?;
    let patches_vec = ps::get_patch_list(repo, &patch_stack)
        .map_err(|e| BranchError::GetPatchListFailed(e.into()))?;

    // validate patch indexes are within bounds
    ps::patch_range_within_stack_bounds(start_patch_index, end_patch_index, &patches_vec)
        .map_err(|e| BranchError::PatchIndexRangeOutOfBounds(e.into()))?;

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
            .map_err(|e| BranchError::GetListPatchInfoFailed(e.into()))?;

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
        new_branch_name = range_patch_branches.first().unwrap().to_string();
        if new_branch_name.starts_with("ps/rr/") && end_patch_index.is_some() {
            return Err(BranchError::PatchSeriesRequireBranchName);
        }
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

    let last_commit_oid_cherry_picked = match end_patch_index {
        Some(end_index) => {
            let end_patch_oid = patches_vec.get(end_index).unwrap().oid;
            cherry_picking::cherry_pick(
                repo,
                &config,
                start_patch_oid,
                Some(end_patch_oid),
                branch_ref_name,
                1,
                false,
                true,
            )
        }
        None => cherry_picking::cherry_pick(
            repo,
            &config,
            start_patch_oid,
            Some(start_patch_oid),
            branch_ref_name,
            1,
            false,
            true,
        ),
    }?
    .expect("No commits cherry picked, when we expected at least one");

    Ok((branch, last_commit_oid_cherry_picked))
}
