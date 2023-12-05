// This is the `ps` module, it is the parenting module collecting all the
// other child Patch Stack specific modules. This module has two
// responsibility, loading it's respective child modules and exposing them
// externally. All code related to these responsibilities belongs here.

pub mod private;
pub mod public;

use std::collections::HashMap;
use std::str::FromStr;

use private::{cherry_picking, git, state_computation};
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
    pub base: git2::Reference<'a>,
}

#[derive(Debug)]
pub enum PatchStackError {
    GitError(git2::Error),
    HeadNoName,
    UpstreamBranchNameNotFound,
}

impl From<git2::Error> for PatchStackError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

impl std::fmt::Display for PatchStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitError(e) => write!(f, "{}", e),
            Self::HeadNoName => write!(f, "head no name"),
            Self::UpstreamBranchNameNotFound => write!(f, "upstream branch name not found"),
        }
    }
}

impl std::error::Error for PatchStackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GitError(e) => Some(e),
            Self::HeadNoName => None,
            Self::UpstreamBranchNameNotFound => None,
        }
    }
}

pub fn get_patch_stack(repo: &git2::Repository) -> Result<PatchStack<'_>, PatchStackError> {
    let head_ref = repo.head()?;
    let repo_gitdir_path = repo.path();

    let head_branch_name = match git::in_rebase(repo_gitdir_path) {
        true => git::in_rebase_head_name(repo_gitdir_path)
            .unwrap()
            .trim()
            .to_string(),
        false => git::get_current_branch(repo).ok_or(PatchStackError::HeadNoName)?,
    };

    let upstream_branch_name_buf = repo
        .branch_upstream_name(&head_branch_name)
        .map_err(PatchStackError::GitError)?;
    let upstream_branch_name = upstream_branch_name_buf
        .as_str()
        .ok_or(PatchStackError::UpstreamBranchNameNotFound)?;
    let base_ref = repo
        .find_reference(upstream_branch_name)
        .map_err(PatchStackError::GitError)?;

    Ok(PatchStack {
        head: head_ref,
        base: base_ref,
    })
}

pub struct ListPatch {
    pub index: usize,
    pub summary: String,
    pub oid: git2::Oid,
}

#[derive(Debug)]
pub enum GetPatchListError {
    CreateRevWalkFailed(git2::Error),
    StackHeadTargetMissing,
    StackBaseTargetMissing,
}

impl std::fmt::Display for GetPatchListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateRevWalkFailed(e) => write!(f, "{}", e),
            Self::StackBaseTargetMissing => write!(f, "Stack base target is missing"),
            Self::StackHeadTargetMissing => write!(f, "Stack head target is missing"),
        }
    }
}

impl std::error::Error for GetPatchListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateRevWalkFailed(e) => Some(e),
            Self::StackBaseTargetMissing => None,
            Self::StackHeadTargetMissing => None,
        }
    }
}

pub fn get_patch_list(
    repo: &git2::Repository,
    patch_stack: &PatchStack,
) -> Result<Vec<ListPatch>, GetPatchListError> {
    let mut rev_walk = repo
        .revwalk()
        .map_err(GetPatchListError::CreateRevWalkFailed)?;
    rev_walk
        .push(
            patch_stack
                .head
                .target()
                .ok_or(GetPatchListError::StackHeadTargetMissing)?,
        )
        .map_err(GetPatchListError::CreateRevWalkFailed)?;
    rev_walk
        .hide(
            patch_stack
                .base
                .target()
                .ok_or(GetPatchListError::StackBaseTargetMissing)?,
        )
        .map_err(GetPatchListError::CreateRevWalkFailed)?;
    rev_walk
        .set_sorting(git2::Sort::REVERSE)
        .map_err(GetPatchListError::CreateRevWalkFailed)?;

    let list_of_patches: Vec<ListPatch> = rev_walk
        .enumerate()
        .map(|(i, rev)| {
            let r = rev.unwrap();
            ListPatch {
                index: i,
                summary: git::get_summary(repo, &r).unwrap(),
                oid: r,
            }
        })
        .collect();
    Ok(list_of_patches)
}

pub fn extract_ps_id(message: &str) -> Option<Uuid> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"ps-id:\s(?P<patchStackId>[\w\d-]+)").unwrap();
    }
    let string = RE
        .captures(message)
        .map(|caps| String::from(&caps["patchStackId"]));
    match string {
        Some(v) => Uuid::from_str(v.as_str()).ok(),
        None => None,
    }
}

pub fn slugify(summary: &str) -> String {
    summary
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase()
}

pub fn generate_rr_branch_name(summary: &str) -> String {
    let slug = slugify(summary);
    format!("ps/rr/{}", slug)
}

#[derive(Debug)]
pub enum AddPatchIdsError {
    GetCurrentBranch,
    FindCurrentBranchReference(Box<dyn std::error::Error>),
    RevParseCurrentBranchReference(Box<dyn std::error::Error>),
    GetCurrentBranchUpstreamName(Box<dyn std::error::Error>),
    RevParseCurrentBranchUpstreamReference(Box<dyn std::error::Error>),
    FindCommonAncestor(Box<dyn std::error::Error>),
    FindCommonAncestorCommit(Box<dyn std::error::Error>),
    CreateAddIdReworkBranch(Box<dyn std::error::Error>),
    GetAddIdReworkBranchReferenceName,
    ConflictsExist(String, String),
    MergeCommitDetected(String),
    CherryPickNoWorkingCopyRange(Box<dyn std::error::Error>),
    SetCurrentBranchTarget(Box<dyn std::error::Error>),
    FindAddIdReworkReference(Box<dyn std::error::Error>),
    DeleteAppIdReworkBranch(Box<dyn std::error::Error>),
}

impl From<cherry_picking::CherryPickError> for AddPatchIdsError {
    fn from(value: cherry_picking::CherryPickError) -> Self {
        match value {
            cherry_picking::CherryPickError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            cherry_picking::CherryPickError::MergeCommitDetected(oid) => {
                Self::MergeCommitDetected(oid)
            }
            _ => Self::CherryPickNoWorkingCopyRange(value.into()),
        }
    }
}

impl std::fmt::Display for AddPatchIdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GetCurrentBranch => write!(f, "failed to get current branch"),
            Self::FindCurrentBranchReference(e) => {
                write!(f, "find current branch reference failed, {}", e)
            }
            Self::RevParseCurrentBranchReference(e) => {
                write!(f, "rev parse current branch reference failed, {}", e)
            }
            Self::GetCurrentBranchUpstreamName(e) => {
                write!(f, "get current branch upstream name failed, {}", e)
            }
            Self::RevParseCurrentBranchUpstreamReference(e) => write!(
                f,
                "rev parse current branch upstream reference failed, {}",
                e
            ),
            Self::FindCommonAncestor(e) => write!(f, "find common ancestor failed, {}", e),
            Self::FindCommonAncestorCommit(e) => {
                write!(f, "find common ancestor commit failed, {}", e)
            }
            Self::CreateAddIdReworkBranch(e) => {
                write!(f, "create add id rework branch failed, {}", e)
            }
            Self::GetAddIdReworkBranchReferenceName => {
                write!(f, "get add id rework branch reference name failed")
            }
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict(s) detected when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::MergeCommitDetected(oid) => {
                write!(f, "merge commit detected with sha {}", oid)
            }
            Self::CherryPickNoWorkingCopyRange(e) => write!(f, "cherry pick failed, {}", e),
            Self::SetCurrentBranchTarget(e) => write!(f, "set current branch target failed, {}", e),
            Self::FindAddIdReworkReference(e) => {
                write!(f, "find add id rework reference failed, {}", e)
            }
            Self::DeleteAppIdReworkBranch(e) => {
                write!(f, "delete app id rework branch failed, {}", e)
            }
        }
    }
}

impl std::error::Error for AddPatchIdsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GetCurrentBranch => None,
            Self::FindCurrentBranchReference(e) => Some(e.as_ref()),
            Self::RevParseCurrentBranchReference(e) => Some(e.as_ref()),
            Self::GetCurrentBranchUpstreamName(e) => Some(e.as_ref()),
            Self::RevParseCurrentBranchUpstreamReference(e) => Some(e.as_ref()),
            Self::FindCommonAncestor(e) => Some(e.as_ref()),
            Self::FindCommonAncestorCommit(e) => Some(e.as_ref()),
            Self::CreateAddIdReworkBranch(e) => Some(e.as_ref()),
            Self::GetAddIdReworkBranchReferenceName => None,
            Self::ConflictsExist(_, _) => None,
            Self::MergeCommitDetected(_) => None,
            Self::CherryPickNoWorkingCopyRange(e) => Some(e.as_ref()),
            Self::SetCurrentBranchTarget(e) => Some(e.as_ref()),
            Self::FindAddIdReworkReference(e) => Some(e.as_ref()),
            Self::DeleteAppIdReworkBranch(e) => Some(e.as_ref()),
        }
    }
}

/// Rebase the currently checked out branch, amending commits with patch identifiers if they are
/// missing.
pub fn add_patch_ids(
    repo: &git2::Repository,
    config: &git2::Config,
) -> Result<(), AddPatchIdsError> {
    // Get currently checked out branch
    let branch_ref_name =
        git::get_current_branch(repo).ok_or(AddPatchIdsError::GetCurrentBranch)?;
    let mut branch_ref = repo
        .find_reference(&branch_ref_name)
        .map_err(|e| AddPatchIdsError::FindCurrentBranchReference(e.into()))?;
    let cur_branch_obj = repo
        .revparse_single(&branch_ref_name)
        .map_err(|e| AddPatchIdsError::RevParseCurrentBranchReference(e.into()))?;
    let cur_branch_oid = cur_branch_obj.id();

    // Get current branches upstream tracking branch
    let upstream_branch_ref_name = git::branch_upstream_name(repo, &branch_ref_name)
        .map_err(|e| AddPatchIdsError::GetCurrentBranchUpstreamName(e.into()))?;
    let upstream_branch_obj = repo
        .revparse_single(&upstream_branch_ref_name)
        .map_err(|e| AddPatchIdsError::RevParseCurrentBranchUpstreamReference(e.into()))?;
    let upstream_branch_oid = upstream_branch_obj.id();

    // find the commmon ancestor
    let common_ancestor_oid = git::common_ancestor(repo, cur_branch_oid, upstream_branch_oid)
        .map_err(|e| AddPatchIdsError::FindCommonAncestor(e.into()))?;
    let common_anccestor_commit = repo
        .find_commit(common_ancestor_oid)
        .map_err(|e| AddPatchIdsError::FindCommonAncestorCommit(e.into()))?;

    // create branch
    let add_id_rework_branch = repo
        .branch("ps/tmp/add_id_rework", &common_anccestor_commit, true)
        .map_err(|e| AddPatchIdsError::CreateAddIdReworkBranch(e.into()))?;
    let add_id_rework_branch_ref_name = add_id_rework_branch
        .get()
        .name()
        .ok_or(AddPatchIdsError::GetAddIdReworkBranchReferenceName)?;

    // cherry pick commits to add_id_rework branch adding patch id if missing
    let last_cherry_picked_commit_oid = cherry_picking::cherry_pick(
        repo,
        config,
        upstream_branch_oid,
        Some(cur_branch_oid),
        add_id_rework_branch_ref_name,
        0,
        true,
        false,
    )?;

    // reset the current branch to point to the add id rework branch head commit
    if let Some(oid) = last_cherry_picked_commit_oid {
        branch_ref
            .set_target(oid, "swap branch to add_id_rework")
            .map_err(|e| AddPatchIdsError::SetCurrentBranchTarget(e.into()))?;
    }

    // delete the add id rework branch
    let mut tmp_branch_ref = repo
        .find_reference(add_id_rework_branch_ref_name)
        .map_err(|e| AddPatchIdsError::FindAddIdReworkReference(e.into()))?;
    tmp_branch_ref
        .delete()
        .map_err(|e| AddPatchIdsError::DeleteAppIdReworkBranch(e.into()))?;

    Ok(())
}

pub fn commit_ps_id(commit: &git2::Commit) -> Option<Uuid> {
    commit.message().and_then(extract_ps_id)
}

#[derive(Debug)]
pub enum PatchRangeWithinStackBoundsError {
    StartPatchIndexOutOfBounds(usize),
    EndPatchIndexOutOfBounds(usize),
}

impl std::fmt::Display for PatchRangeWithinStackBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartPatchIndexOutOfBounds(idx) => {
                write!(f, "start patch index ({}) out of bounds", idx)
            }
            Self::EndPatchIndexOutOfBounds(idx) => {
                write!(f, "end patch index ({}) out of bounds", idx)
            }
        }
    }
}

impl std::error::Error for PatchRangeWithinStackBoundsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StartPatchIndexOutOfBounds(_) => None,
            Self::EndPatchIndexOutOfBounds(_) => None,
        }
    }
}

pub fn patch_range_within_stack_bounds(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    stack_patches: &Vec<ListPatch>,
) -> Result<(), PatchRangeWithinStackBoundsError> {
    if start_patch_index > (stack_patches.len() - 1) {
        return Err(
            PatchRangeWithinStackBoundsError::StartPatchIndexOutOfBounds(start_patch_index),
        );
    }

    if let Some(end_index) = end_patch_index {
        if end_index > (stack_patches.len() - 1) {
            return Err(PatchRangeWithinStackBoundsError::EndPatchIndexOutOfBounds(
                end_index,
            ));
        }
    }

    Ok(())
}

/// Get a vec of all the unique branch names associated with specified patch series
pub fn patch_series_unique_branch_names(
    repo: &git2::Repository,
    stack_patches: &[ListPatch],
    patch_info_collection: &HashMap<Uuid, state_computation::PatchGitInfo>,
    start_patch_index: usize,
    end_patch_index: Option<usize>,
) -> Vec<String> {
    // collect vector of indexes
    let indexes_iter = match end_patch_index {
        Some(end_index) => start_patch_index..=end_index,
        None => start_patch_index..=start_patch_index,
    };

    // get unique branch names of patches in series
    let mut range_patch_branches: Vec<String> = indexes_iter
        .clone()
        .map(|i| stack_patches.get(i).unwrap())
        .filter_map(|lp| {
            let commit = repo.find_commit(lp.oid).unwrap();
            commit_ps_id(&commit)
        })
        .filter_map(|id| patch_info_collection.get(&id))
        .flat_map(|pi| pi.branches.iter().map(|b| b.name.clone()))
        .collect();
    range_patch_branches.sort();
    range_patch_branches.dedup();

    range_patch_branches
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_extract_ps_id_with_ps_id() {
        let msg = "Some summary\n\nSome paragraph\nSome more lines of the paragraph\n <!-- ps-id: 2dce2a21-72b9-487a-b641-4a0b157b76e8 -->\n some other stuff";
        let opt = super::extract_ps_id(msg);
        assert!(opt.is_some());
        assert_eq!(
            opt.unwrap(),
            Uuid::from_str("2dce2a21-72b9-487a-b641-4a0b157b76e8").unwrap()
        );
    }

    #[test]
    fn test_extract_ps_id_without_ps_id() {
        let msg = "Some summary\n\nSome paragraph\nSome more lines of the paragraph\n aeuae uaeou aoeu aoeeo\n some other stuff";
        let opt = super::extract_ps_id(msg);
        assert!(opt.is_none());
    }

    #[test]
    fn test_slugify() {
        assert_eq!(
            super::slugify("Hello & Goodbye - Purple %#@!()"),
            "hello___goodbye___purple_______"
        );
    }

    #[test]
    fn test_generate_rr_branch_name() {
        assert_eq!(
            super::generate_rr_branch_name("Hello & Goodbye"),
            "ps/rr/hello___goodbye"
        );
    }
}
