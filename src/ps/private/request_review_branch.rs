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
        }
    }
}

pub fn request_review_branch(
    repo: &git2::Repository,
    patch_index: usize,
    given_branch_name_option: Option<String>,
) -> Result<(git2::Branch<'_>, Uuid, git2::Oid), RequestReviewBranchError> {
    let config =
        git2::Config::open_default().map_err(RequestReviewBranchError::OpenGitConfigFailed)?;

    // - find the patch identified by the patch_index
    let patch_stack = ps::get_patch_stack(repo)?;
    let patches_vec = ps::get_patch_list(repo, &patch_stack)
        .map_err(RequestReviewBranchError::GetPatchListFailed)?;
    let patch_oid = patches_vec
        .get(patch_index)
        .ok_or(RequestReviewBranchError::PatchIndexNotFound)?
        .oid;
    let patch_commit = repo
        .find_commit(patch_oid)
        .map_err(|_| RequestReviewBranchError::PatchCommitNotFound)?;
    let patch_message = patch_commit
        .message()
        .ok_or(RequestReviewBranchError::PatchMessageMissing)?;

    // fetch or add patch id given patch_message
    let new_patch_oid: git2::Oid;
    let ps_id: Uuid;
    if let Some(extracted_ps_id) = ps::extract_ps_id(patch_message) {
        ps_id = extracted_ps_id;
        new_patch_oid = patch_oid;
    } else {
        ps_id = Uuid::new_v4();
        new_patch_oid = ps::add_ps_id(repo, &config, patch_oid, ps_id)?;
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

    // use provided branch name, or fall back to patch associated branch name, or fall back to
    // generated branch name
    let branch_name: String = match patch_info_collection.get(&ps_id) {
        Some(patch_info) => {
            if patch_info.branches.len() == 1 {
                Ok(patch_info.branches.first().unwrap().name.clone())
            } else {
                Err(RequestReviewBranchError::PatchBranchAmbiguous)
            }
        }
        None => {
            let patch_summary = patch_commit
                .summary()
                .ok_or(RequestReviewBranchError::PatchSummaryMissing)?;
            let default_branch_name = ps::generate_rr_branch_name(patch_summary);
            Ok(given_branch_name_option.unwrap_or(default_branch_name))
        }
    }?;

    // create branch on top of the patch stack base
    let branch = repo
        .branch(branch_name.as_str(), &patch_stack_base_commit, true)
        .map_err(|_| RequestReviewBranchError::CreateRrBranchFailed)?;

    let branch_ref_name = branch
        .get()
        .name()
        .ok_or(RequestReviewBranchError::RrBranchNameNotUtf8)?;

    // cherry pick the patch onto new rr branch with commiter timestamp offset
    // 1 second into the future so that it doesn't overlap with the
    // add_ps_id()'s cherry pick committer timestamp
    let new_commit_oid =
        git::cherry_pick_no_working_copy(repo, &config, new_patch_oid, branch_ref_name, 1)
            .map_err(RequestReviewBranchError::CherryPickFailed)?;

    Ok((branch, ps_id, new_commit_oid))
}
