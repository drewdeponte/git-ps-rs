use super::super::super::ps;
use super::super::private;
use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum AppendError {
    OpenRepositoryFailed(Box<dyn std::error::Error>),
    OpenGitConfigFailed(Box<dyn std::error::Error>),
    FindBranchFailed(Box<dyn std::error::Error>),
    BranchNameNotUtf8,
    GetPatchStackFailed(Box<dyn std::error::Error>),
    GetPatchListFailed(Box<dyn std::error::Error>),
    FailedToMapIndexesForCherryPick(Box<dyn std::error::Error>),
    AddPatchIdsFailed(Box<dyn std::error::Error>),
    CherryPickFailed(Box<dyn std::error::Error>),
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for AppendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenRepositoryFailed(e) => write!(f, "failed to open repository {}", e),
            Self::OpenGitConfigFailed(e) => write!(f, "failed to open git config, {}", e),
            Self::FindBranchFailed(e) => {
                write!(f, "failed to find branch with the provided name {}", e)
            }
            Self::BranchNameNotUtf8 => write!(f, "matching branch name isn't utf8"),
            Self::GetPatchStackFailed(e) => write!(f, "failed to get patch stack, {}", e),
            Self::GetPatchListFailed(e) => write!(f, "failed to get patch list, {}", e),
            Self::FailedToMapIndexesForCherryPick(e) => {
                write!(f, "failed to map indexes for cherry pick {}", e)
            }
            Self::AddPatchIdsFailed(e) => write!(f, "failed to add patch ids, {}", e),
            Self::CherryPickFailed(e) => write!(f, "failed to cherry pick, {}", e),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenRepositoryFailed(e) => Some(e.as_ref()),
            Self::OpenGitConfigFailed(e) => Some(e.as_ref()),
            Self::FindBranchFailed(e) => Some(e.as_ref()),
            Self::BranchNameNotUtf8 => None,
            Self::GetPatchStackFailed(e) => Some(e.as_ref()),
            Self::GetPatchListFailed(e) => Some(e.as_ref()),
            Self::FailedToMapIndexesForCherryPick(e) => Some(e.as_ref()),
            Self::AddPatchIdsFailed(e) => Some(e.as_ref()),
            Self::CherryPickFailed(e) => Some(e.as_ref()),
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn append(
    start_patch_index: usize,
    end_patch_index_optional: Option<usize>,
    branch_name: String,
) -> Result<(), AppendError> {
    let repo = git::create_cwd_repo().map_err(|e| AppendError::OpenRepositoryFailed(e.into()))?;

    let config =
        git2::Config::open_default().map_err(|e| AppendError::OpenGitConfigFailed(e.into()))?;

    let branch = repo
        .find_branch(&branch_name, git2::BranchType::Local)
        .map_err(|e| AppendError::FindBranchFailed(e.into()))?;
    let branch_ref_name = branch.get().name().ok_or(AppendError::BranchNameNotUtf8)?;

    ps::add_patch_ids(&repo, &config).map_err(|e| AppendError::AddPatchIdsFailed(e.into()))?;

    let patch_stack =
        ps::get_patch_stack(&repo).map_err(|e| AppendError::GetPatchStackFailed(e.into()))?;
    let patches_vec = ps::get_patch_list(&repo, &patch_stack)
        .map_err(|e| AppendError::GetPatchListFailed(e.into()))?;

    let cherry_pick_range = private::cherry_picking::map_range_for_cherry_pick(
        &patches_vec,
        start_patch_index,
        end_patch_index_optional,
    )
    .map_err(|e| AppendError::FailedToMapIndexesForCherryPick(e.into()))?;

    private::cherry_picking::cherry_pick(
        &repo,
        &config,
        cherry_pick_range.root_oid,
        cherry_pick_range.leaf_oid,
        branch_ref_name,
        1,
        false,
        true,
    )
    .map_err(|e| AppendError::CherryPickFailed(e.into()))?;

    Ok(())
}
