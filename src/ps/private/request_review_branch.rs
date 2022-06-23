use super::super::private::git;
use super::super::private::state_management;
use super::super::super::ps;
use uuid::Uuid;
use std::result::Result;
use std::fmt;
use super::paths;

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
  ReadPatchMetaDataFailed(state_management::ReadPatchStatesError),
  WritePatchMetaDataFailed(state_management::WritePatchStatesError),
  OpenGitConfigFailed(git2::Error),
  PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError)
}

impl From<git::CreateCwdRepositoryError> for RequestReviewBranchError {
  fn from(_e: git::CreateCwdRepositoryError) -> Self {
    RequestReviewBranchError::RepositoryMissing
  }
}

impl From<ps::PatchStackError> for RequestReviewBranchError {
  fn from(e: ps::PatchStackError) -> Self {
    match e {
      ps::PatchStackError::GitError(_git2_error) => RequestReviewBranchError::PatchStackNotFound,
      ps::PatchStackError::HeadNoName => RequestReviewBranchError::PatchStackNotFound,
      ps::PatchStackError::UpstreamBranchNameNotFound => RequestReviewBranchError::PatchStackNotFound,
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
      RequestReviewBranchError::RepositoryMissing => write!(f, "Repository not found in current working directory"),
      RequestReviewBranchError::PatchStackNotFound => write!(f, "Patch Stack not found"),
      RequestReviewBranchError::PatchStackBaseNotFound => write!(f, "Patch Stack Base not found"),
      RequestReviewBranchError::PatchIndexNotFound => write!(f, "Patch Index out of range"),
      RequestReviewBranchError::PatchCommitNotFound => write!(f, "Patch commit not found"),
      RequestReviewBranchError::PatchMessageMissing => write!(f, "Patch missing message"),
      RequestReviewBranchError::AddPsIdToPatchFailed(_add_ps_id_error) => write!(f, "Failed to add patch stack id to patch"),
      RequestReviewBranchError::PatchSummaryMissing => write!(f, "Patch missing summary"),
      RequestReviewBranchError::CreateRrBranchFailed => write!(f, "Failed to create request-review branch"),
      RequestReviewBranchError::RrBranchNameNotUtf8 => write!(f, "request-review branch is not utf8"),
      RequestReviewBranchError::CherryPickFailed(_git_error) => write!(f, "Failed to cherry pick"),
      RequestReviewBranchError::GetPatchListFailed(_patch_list_error) => write!(f, "Failed to get patch list"),
      RequestReviewBranchError::GetPatchMetaDataPathFailed(_patch_meta_data_path_error) => write!(f, "Failed to get patch meta data path {:?}", _patch_meta_data_path_error),
      RequestReviewBranchError::ReadPatchMetaDataFailed(_read_patch_meta_data_error) => write!(f, "Failed to read patch meta data {:?}", _read_patch_meta_data_error),
      RequestReviewBranchError::WritePatchMetaDataFailed(_write_patch_meta_data_error) => write!(f, "Failed to write patch meta data {:?}", _write_patch_meta_data_error),
      RequestReviewBranchError::OpenGitConfigFailed(_) => write!(f, "Failed to open git config"),
      RequestReviewBranchError::PatchCommitDiffPatchIdFailed(_) => write!(f, "Failed to get commit diff patch id")
    }
  }
}


pub fn request_review_branch(repo: &git2::Repository, patch_index: usize, given_branch_name_option: Option<String>) -> Result<(git2::Branch<'_>, Uuid), RequestReviewBranchError>  {
  let config = git2::Config::open_default().map_err(RequestReviewBranchError::OpenGitConfigFailed)?;

  // - find the patch identified by the patch_index
  let patch_stack = ps::get_patch_stack(repo)?;
  let patch_stack_base_commit = patch_stack.base.peel_to_commit().map_err(|_| RequestReviewBranchError::PatchStackBaseNotFound)?;
  let patches_vec = ps::get_patch_list(repo, &patch_stack).map_err(RequestReviewBranchError::GetPatchListFailed)?;
  let patch_oid = patches_vec.get(patch_index).ok_or(RequestReviewBranchError::PatchIndexNotFound)?.oid;
  let patch_commit = repo.find_commit(patch_oid).map_err(|_| RequestReviewBranchError::PatchCommitNotFound)?;
  let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).map_err(RequestReviewBranchError::PatchCommitDiffPatchIdFailed)?;

  let patch_message = patch_commit.message().ok_or(RequestReviewBranchError::PatchMessageMissing)?;

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

  // fetch patch meta data given repo and patch_id
  let patch_meta_data_path = paths::patch_states_path(repo).map_err(RequestReviewBranchError::GetPatchMetaDataPathFailed)?;
  let mut patch_meta_data = state_management::read_patch_states(&patch_meta_data_path).map_err(RequestReviewBranchError::ReadPatchMetaDataFailed)?;
  let branch_name = match patch_meta_data.get(&ps_id) {
    Some(patch_meta_data) => patch_meta_data.state.branch_name(),
    None => {
      let patch_summary = patch_commit.summary().ok_or(RequestReviewBranchError::PatchSummaryMissing)?;
      let default_branch_name = ps::generate_rr_branch_name(patch_summary);
      given_branch_name_option.unwrap_or(default_branch_name)
    }
  };

  let branch = repo.branch(branch_name.as_str(), &patch_stack_base_commit, true).map_err(|_| RequestReviewBranchError::CreateRrBranchFailed)?;
  
  let branch_ref_name = branch.get().name().ok_or(RequestReviewBranchError::RrBranchNameNotUtf8)?;

  // - cherry pick the patch onto new rr branch
  git::cherry_pick_no_working_copy(repo, &config, new_patch_oid, branch_ref_name, 0).map_err(RequestReviewBranchError::CherryPickFailed)?;

  // record patch state if there is no record
  if patch_meta_data.get(&ps_id).is_none() {
    let new_patch_meta_data = state_management::Patch {
      patch_id: ps_id,
      state: state_management::PatchState::BranchCreated(branch_name, patch_commit_diff_patch_id.to_string())
    };
    patch_meta_data.insert(ps_id, new_patch_meta_data);
    state_management::write_patch_states(&patch_meta_data_path, &patch_meta_data).map_err(RequestReviewBranchError::WritePatchMetaDataFailed)?;
  }

  Ok((branch, ps_id))
}
