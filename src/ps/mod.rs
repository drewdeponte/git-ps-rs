// This is the `ps` module, it is the parenting module collecting all the
// other child Patch Stack specific modules. This module has two
// responsibility, loading it's respective child modules and exposing them
// externally. All code related to these responsibilities belongs here.


pub mod public;
pub mod private;

use std::str::FromStr;

use private::git;
use private::state_management;
use private::paths;
// This is the `ps` module. It is responsible for housing functionality
// specific to Patch Stack as a conceptual level.  It is responsible for
// consuming functionality from other modules like the `git` and `utils`
// modules to build a higher level abstraction around the concepts of Patch
// Stack. Think of this as the public interface for a Patch Stack library that
// will be consumed by each of the subcommand specific modules.
//
// All code fitting that description belongs here.

use regex::Regex;
use uuid::Uuid;

pub struct PatchStack<'a> {
    pub head: git2::Reference<'a>,
    pub base: git2::Reference<'a>
}

#[derive(Debug)]
pub enum PatchStackError {
    GitError(git2::Error),
    HeadNoName,
    UpstreamBranchNameNotFound
}

impl From<git2::Error> for PatchStackError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

pub fn get_patch_stack<'a>(repo: &'a git2::Repository) -> Result<PatchStack<'a>, PatchStackError> {
    let head_ref = repo.head()?;
    let upstream_branch_name_buf = head_ref.name().ok_or(PatchStackError::HeadNoName)
        .and_then(|head_branch_name| repo.branch_upstream_name(head_branch_name).map_err(PatchStackError::GitError))?;
    let upstream_branch_name = upstream_branch_name_buf.as_str().ok_or(PatchStackError::UpstreamBranchNameNotFound)?;
    let base_ref = repo.find_reference(upstream_branch_name).map_err(PatchStackError::GitError)?;

    Ok(PatchStack { head: head_ref, base: base_ref })
}

pub struct ListPatch {
    pub index: usize,
    pub summary: String,
    pub oid: git2::Oid
}

#[derive(Debug)]
pub enum GetPatchListError {
  CreateRevWalkFailed(git2::Error),
  StackHeadTargetMissing,
  StackBaseTargetMissing
}

pub fn get_patch_list(repo: &git2::Repository, patch_stack: PatchStack) -> Result<Vec<ListPatch>, GetPatchListError> {
    let mut rev_walk = repo.revwalk().map_err(|e| GetPatchListError::CreateRevWalkFailed(e))?;
    rev_walk.push(patch_stack.head.target().ok_or(GetPatchListError::StackHeadTargetMissing)?).map_err(|e| GetPatchListError::CreateRevWalkFailed(e))?;
    rev_walk.hide(patch_stack.base.target().ok_or(GetPatchListError::StackBaseTargetMissing)?).map_err(|e| GetPatchListError::CreateRevWalkFailed(e))?;
    rev_walk.set_sorting(git2::Sort::REVERSE).map_err(|e| GetPatchListError::CreateRevWalkFailed(e))?;

    let list_of_patches: Vec<ListPatch> = rev_walk.enumerate().map(|(i, rev)| {
        let r = rev.unwrap();
        ListPatch { index: i, summary: git::get_summary(&repo, &r).unwrap(), oid: r }
    }).collect();
    return Ok(list_of_patches);
}

pub fn extract_ps_id(message: &str) -> Option<Uuid> {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"ps-id:\s(?P<patchStackId>[\w\d-]+)").unwrap();
  }
  let string = RE.captures(message).map(|caps| String::from(&caps["patchStackId"]));
  match string {
    Some(v) => Uuid::from_str(v.as_str()).ok(),
    None => None
  }
}

pub fn slugify(summary: &str) -> String {
  return summary.replace(|c: char| !c.is_alphanumeric(), "_").to_lowercase();
}

pub fn generate_rr_branch_name(summary: &str) -> String {
  let slug = slugify(summary);
  return format!("ps/rr/{}", slug);
}

#[derive(Debug)]
pub enum AddPsIdError {
  GitError(git2::Error),
  FailedToGetCurrentBranch,
  UpstreamBranchNotFound,
  FailedToGetReferenceName,
  TargetNotFound,
  ReferenceNameMissing,
  CommitMessageMissing
}

impl From<git2::Error> for AddPsIdError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

impl From<git::GitError> for AddPsIdError {
    fn from(e: git::GitError) -> Self {
      match e {
        git::GitError::NotFound => AddPsIdError::UpstreamBranchNotFound,
        git::GitError::GitError(err) => AddPsIdError::GitError(err),
        git::GitError::TargetNotFound => AddPsIdError::TargetNotFound,
        git::GitError::ReferenceNameMissing => AddPsIdError::ReferenceNameMissing,
        git::GitError::CommitMessageMissing => AddPsIdError::CommitMessageMissing
      }
    }
}

pub fn add_ps_id(repo: &git2::Repository, commit_oid: git2::Oid, ps_id: Uuid) -> Result<git2::Oid, AddPsIdError> {
  // Get currently checked out branch
  let branch_ref_name = git::get_current_branch(&repo).ok_or(AddPsIdError::FailedToGetCurrentBranch)?;
  let mut branch_ref = repo.find_reference(&branch_ref_name)?;
  let cur_branch_obj = repo.revparse_single(&branch_ref_name)?;
  let cur_branch_oid = cur_branch_obj.id();

  // Get current branches upstream tracking branch
  let upstream_branch_ref_name = git::branch_upstream_name(&repo, &branch_ref_name)?;
  let upstream_branch_obj = repo.revparse_single(&upstream_branch_ref_name)?;
  let upstream_branch_oid = upstream_branch_obj.id();
  let upstream_branch_commit = repo.find_commit(upstream_branch_oid)?;

  // create branch
  let add_id_rework_branch = repo.branch("ps/tmp/add_id_rework", &upstream_branch_commit, true)?;
  let add_id_rework_branch_ref_name = add_id_rework_branch.get().name().ok_or(AddPsIdError::FailedToGetReferenceName)?;

  // cherry pick
  git::cherry_pick_no_working_copy_range(&repo, commit_oid, upstream_branch_oid, add_id_rework_branch_ref_name)?;

  let message_amendment = format!("\nps-id: {}", ps_id.to_hyphenated().to_string());
  let amended_patch_oid = git::cherry_pick_no_working_copy_amend_message(&repo, commit_oid, add_id_rework_branch_ref_name, message_amendment.as_str())?;

  if cur_branch_oid != commit_oid {
    git::cherry_pick_no_working_copy_range(&repo, cur_branch_oid, commit_oid, add_id_rework_branch_ref_name)?;
    let cherry_picked_commit_oid = git::cherry_pick_no_working_copy(&repo, cur_branch_oid, add_id_rework_branch_ref_name)?;
    branch_ref.set_target(cherry_picked_commit_oid, "swap branch to add_id_rework")?;
  } else {
    branch_ref.set_target(amended_patch_oid, "swap branch to add_id_rework")?;
  }

  // delete temporary branch
  let mut tmp_branch_ref = repo.find_reference(add_id_rework_branch_ref_name)?;
  tmp_branch_ref.delete()?;

  Ok(amended_patch_oid)
}

#[derive(Debug)]
pub enum FindPatchCommitError {
  GetPatchStackDescFailed(PatchStackError),
  GetPatchListFailed(GetPatchListError),
  PatchWithIndexNotFound(usize),
  FindCommitWithOidFailed(git2::Oid, git2::Error)
}

pub fn find_patch_commit(repo: &git2::Repository, patch_index: usize) -> Result<git2::Commit, FindPatchCommitError> {
  let patch_stack = get_patch_stack(repo).map_err(FindPatchCommitError::GetPatchStackDescFailed)?;
  let patches_vec = get_patch_list(repo, patch_stack).map_err(FindPatchCommitError::GetPatchListFailed)?;
  let patch_oid = patches_vec.get(patch_index).ok_or(FindPatchCommitError::PatchWithIndexNotFound(patch_index))?.oid;
  repo.find_commit(patch_oid).map_err(|e| FindPatchCommitError::FindCommitWithOidFailed(patch_oid, e))
}

pub fn commit_ps_id(commit: &git2::Commit) -> Option<Uuid> {
  commit.message().and_then(extract_ps_id)
}

#[derive(Debug)]
pub enum GetPatchMetaDataError {
  GetPatchStatesPatchFailed(paths::PathsError),
  ReadPatchStatesFailed(state_management::ReadPatchStatesError)
}

pub fn get_patch_meta_data(repo: &git2::Repository, ps_id: Uuid) -> Result<Option<state_management::Patch>, GetPatchMetaDataError> {
  let patch_meta_data_path = paths::patch_states_path(repo).map_err(GetPatchMetaDataError::GetPatchStatesPatchFailed)?;
  let patch_meta_data = state_management::read_patch_states(&patch_meta_data_path).map_err(GetPatchMetaDataError::ReadPatchStatesFailed)?;
  Ok(patch_meta_data.get(&ps_id).cloned())
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;
  use std::str::FromStr;

  #[test]
  fn test_extract_ps_id_with_ps_id() {
    let msg = "Some summary\n\nSome paragraph\nSome more lines of the paragraph\n      ps-id: 2dce2a21-72b9-487a-b641-4a0b157b76e8\n some other stuff";
    let opt = super::extract_ps_id(&msg);
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), Uuid::from_str("2dce2a21-72b9-487a-b641-4a0b157b76e8").unwrap());
  }

  #[test]
  fn test_extract_ps_id_without_ps_id() {
    let msg = "Some summary\n\nSome paragraph\nSome more lines of the paragraph\n aeuae uaeou aoeu aoeeo\n some other stuff";
    let opt = super::extract_ps_id(&msg);
    assert!(opt.is_none());
  }

  #[test]
  fn test_slugify() {
    assert_eq!(super::slugify("Hello & Goodbye - Purple %#@!()"), "hello___goodbye___purple_______");
  }

  #[test]
  fn test_generate_rr_branch_name() {
    assert_eq!(super::generate_rr_branch_name("Hello & Goodbye"), "ps/rr/hello___goodbye");
  }
}
