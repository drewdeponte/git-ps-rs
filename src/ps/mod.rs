// This is the `ps` module, it is the parenting module collecting all the
// other child Patch Stack specific modules. This module has two
// responsibility, loading it's respective child modules and exposing them
// externally. All code related to these responsibilities belongs here.

pub mod private;
pub mod public;

use std::collections::HashMap;
use std::str::FromStr;

use private::{git, state_computation};
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

pub fn get_patch_stack(repo: &git2::Repository) -> Result<PatchStack<'_>, PatchStackError> {
    let head_ref = repo.head()?;
    let upstream_branch_name_buf = head_ref
        .name()
        .ok_or(PatchStackError::HeadNoName)
        .and_then(|head_branch_name| {
            repo.branch_upstream_name(head_branch_name)
                .map_err(PatchStackError::GitError)
        })?;
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

pub fn generate_branch_branch_name(summary: &str) -> String {
    let slug = slugify(summary);
    format!("ps/branch/{}", slug)
}

/// Cherry pick either an individual commit identified by the `root_oid` Oid and None for
/// `leaf_oid`, or a range of commits identified by the `root_oid` and `leaf_oid` both having Oids.
///
/// The given `repo` is the repository that you want to cherry pick the range of commits within.
/// The `config` is the config used to facilitate commit creation, providing things like the
/// author, email, etc.
///
/// The `root_oid` specifies the commit to start the ranged cherry picking process from,
/// inclusively. Meaning this commit WILL be included in the cherry picked commits, as will all its
/// descendants up to and including the `leaf_oid`. This commit should be an ancestor to the
/// `leaf_oid`.
///
/// The `leaf_oid` specifies the commit to end the ranged cherry picking process on, inclusively.
/// Meaning this commit will be included in the cherry picked commits. This commit should be a
/// descendant to the `root_oid`.
///
/// The `dest_ref_name` specifies the reference (e.g. branch) to cherry pick the range of commits
/// into.
///
/// It returns an Ok(Option(last_cherry_picked_commit_oid)) result in the case of success and an
/// error result of GitError in the case of failure.
pub fn cherry_pick(
    repo: &'_ git2::Repository,
    config: &git2::Config,
    root_oid: git2::Oid,
    leaf_oid: Option<git2::Oid>,
    dest_ref_name: &str,
    add_missing_patch_ids: bool,
) -> Result<Option<git2::Oid>, git::GitError> {
    Ok(match leaf_oid {
        Some(leaf_oid) => {
            let root_commit = repo.find_commit(root_oid)?;
            let root_commit_parent_commit = root_commit.parent(0)?;
            let root_commit_parent_commit_oid = root_commit_parent_commit.id();
            cherry_pick_no_working_copy_range(
                repo,
                config,
                root_commit_parent_commit_oid,
                leaf_oid,
                dest_ref_name,
                0,
                add_missing_patch_ids,
            )?
        }
        None => Some(cherry_pick_no_working_copy(
            repo,
            config,
            root_oid,
            dest_ref_name,
            0,
            add_missing_patch_ids,
        )?),
    })
}

/// Cherry pick the specified range of commits onto the destination ref
///
/// The given `repo` is the repository that you want to cherry pick the range of commits within.
/// The `config` is the config used to facilitate commit creation, providing things like the
/// author, email, etc.
///
/// The `root_oid` specifies the commit to start the ranged cherry picking process from,
/// exclusively. Meaning this commit won't be included in the cherry picked commits, but its
/// descendants will be, up to and including the `leaf_oid`. This commit should be an ancestor to
/// the `leaf_oid`.
///
/// The `leaf_oid` specifies the commit to end the ranged cherry picking process on, inclusively.
/// Meaning this commit will be included in the cherry picked commits. This commit should be a
/// descendant to the `root_oid`.
///
/// The `dest_ref_name` specifies the reference (e.g. branch) to cherry pick the range of commits
/// into.
///
/// It returns an Ok(Option(last_cherry_picked_commit_oid)) result in the case of success and an
/// error result of GitError in the case of failure.
pub fn cherry_pick_no_working_copy_range(
    repo: &'_ git2::Repository,
    config: &git2::Config,
    root_oid: git2::Oid,
    leaf_oid: git2::Oid,
    dest_ref_name: &str,
    committer_time_offset: i64,
    add_missing_patch_ids: bool,
) -> Result<Option<git2::Oid>, git::GitError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push(leaf_oid)?; // start traversal from leaf_oid and walk to root_oid
    rev_walk.hide(root_oid)?; // mark root_oid as where to hide from
    rev_walk.set_sorting(git2::Sort::REVERSE)?; // reverse traversal order so we walk from child
                                                // commit of the commit identified by root_oid and
                                                // then iterate our way to the the commit
                                                // identified by the leaf_oid

    let mut last_cherry_picked_oid: Option<git2::Oid> = None;

    for rev in rev_walk.flatten() {
        last_cherry_picked_oid = Some(cherry_pick_no_working_copy(
            repo,
            config,
            rev,
            dest_ref_name,
            committer_time_offset,
            add_missing_patch_ids,
        )?);
    }

    Ok(last_cherry_picked_oid)
}

/// Cherry pick the commit identified by the oid to the dest_ref_name with the
/// given committer_time_offset. Note: The committer_time_offset is used to
/// offset the Commiter's signature timestamp which is in seconds since epoch
/// so that if we are performing multiple operations on the same commit within
/// less than a second we can offset it in one direction or the other. The
/// current use case for this is when we add patch stack id to a commit and
/// then immediately cherry pick that commit into the ps/rr/whatever branch as
/// part of the request_review_branch() operation.
pub fn cherry_pick_no_working_copy<'a>(
    repo: &'a git2::Repository,
    config: &'a git2::Config,
    oid: git2::Oid,
    dest_ref_name: &str,
    committer_time_offset: i64,
    add_missing_patch_id: bool,
) -> Result<git2::Oid, git::GitError> {
    // https://www.pygit2.org/recipes/git-cherry-pick.html#cherry-picking-a-commit-without-a-working-copy
    let commit = repo.find_commit(oid)?;
    let commit_tree = commit.tree()?;

    let commit_parent = commit.parent(0)?;
    let commit_parent_tree = commit_parent.tree()?;

    let destination_ref = repo.find_reference(dest_ref_name)?;
    let destination_oid = destination_ref
        .target()
        .ok_or(git::GitError::TargetNotFound)?;

    // let common_ancestor_oid = repo.merge_base(oid, destination_oid)?;
    // let common_ancestor_commit = repo.find_commit(common_ancestor_oid)?;
    // let common_ancestor_tree = common_ancestor_commit.tree()?;

    let destination_commit = repo.find_commit(destination_oid)?;
    let destination_tree = destination_commit.tree()?;

    let mut index = repo.merge_trees(&commit_parent_tree, &destination_tree, &commit_tree, None)?;
    let tree_oid = index.write_tree_to(repo)?;
    let tree = repo.find_tree(tree_oid)?;

    let author = commit.author();
    let committer = repo.signature().unwrap();

    let message = commit.message().unwrap();

    let new_time = git2::Time::new(
        committer.when().seconds() + committer_time_offset,
        committer.when().offset_minutes(),
    );
    let new_committer = git2::Signature::new(
        committer.name().unwrap(),
        committer.email().unwrap(),
        &new_time,
    )
    .unwrap();

    let possibly_amended_mesesage = match add_missing_patch_id {
        true => match commit_ps_id(&commit) {
            Some(_) => message.to_string(),
            None => {
                let patch_id: uuid::Uuid = uuid::Uuid::new_v4();
                let message_amendment = format!("\n<!-- ps-id: {} -->", patch_id.hyphenated());
                format!("{}{}", message, message_amendment)
            }
        },
        false => message.to_string(),
    };

    let new_commit_oid = git::create_commit(
        repo,
        config,
        dest_ref_name,
        &author,
        &new_committer,
        &possibly_amended_mesesage,
        &tree,
        &[&destination_commit],
    )
    .unwrap();

    Ok(new_commit_oid)
}

#[derive(Debug)]
pub enum AddPatchIdsError {
    GetCurrentBranch,
    FindCurrentBranchReference(git2::Error),
    RevParseCurrentBranchReference(git2::Error),
    GetCurrentBranchUpstreamName(git::GitError),
    RevParseCurrentBranchUpstreamReference(git2::Error),
    FindCommonAncestor(git::CommonAncestorError),
    FindCommonAncestorCommit(git2::Error),
    CreateAddIdReworkBranch(git2::Error),
    GetAddIdReworkBranchReferenceName,
    CherryPickNoWorkingCopyRange(git::GitError),
    SetCurrentBranchTarget(git2::Error),
    FindAddIdReworkReference(git2::Error),
    DeleteAppIdReworkBranch(git2::Error),
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
        .map_err(AddPatchIdsError::FindCurrentBranchReference)?;
    let cur_branch_obj = repo
        .revparse_single(&branch_ref_name)
        .map_err(AddPatchIdsError::RevParseCurrentBranchReference)?;
    let cur_branch_oid = cur_branch_obj.id();

    // Get current branches upstream tracking branch
    let upstream_branch_ref_name = git::branch_upstream_name(repo, &branch_ref_name)
        .map_err(AddPatchIdsError::GetCurrentBranchUpstreamName)?;
    let upstream_branch_obj = repo
        .revparse_single(&upstream_branch_ref_name)
        .map_err(AddPatchIdsError::RevParseCurrentBranchUpstreamReference)?;
    let upstream_branch_oid = upstream_branch_obj.id();

    // find the commmon ancestor
    let common_ancestor_oid = git::common_ancestor(repo, cur_branch_oid, upstream_branch_oid)
        .map_err(AddPatchIdsError::FindCommonAncestor)?;
    let common_anccestor_commit = repo
        .find_commit(common_ancestor_oid)
        .map_err(AddPatchIdsError::FindCommonAncestorCommit)?;

    // create branch
    let add_id_rework_branch = repo
        .branch("ps/tmp/add_id_rework", &common_anccestor_commit, true)
        .map_err(AddPatchIdsError::CreateAddIdReworkBranch)?;
    let add_id_rework_branch_ref_name = add_id_rework_branch
        .get()
        .name()
        .ok_or(AddPatchIdsError::GetAddIdReworkBranchReferenceName)?;

    // cherry pick commits to add_id_rework branch adding patch id if missing
    let last_cherry_picked_commit_oid = cherry_pick_no_working_copy_range(
        repo,
        config,
        upstream_branch_oid,
        cur_branch_oid,
        add_id_rework_branch_ref_name,
        0,
        true,
    )
    .map_err(AddPatchIdsError::CherryPickNoWorkingCopyRange)?;

    // reset the current branch to point to the add id rework branch head commit
    if let Some(oid) = last_cherry_picked_commit_oid {
        branch_ref
            .set_target(oid, "swap branch to add_id_rework")
            .map_err(AddPatchIdsError::SetCurrentBranchTarget)?;
    }

    // delete the add id rework branch
    let mut tmp_branch_ref = repo
        .find_reference(add_id_rework_branch_ref_name)
        .map_err(AddPatchIdsError::FindAddIdReworkReference)?;
    tmp_branch_ref
        .delete()
        .map_err(AddPatchIdsError::DeleteAppIdReworkBranch)?;

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
    stack_patches: &Vec<ListPatch>,
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
        .map(|lp| {
            let commit = repo.find_commit(lp.oid).unwrap();
            commit_ps_id(&commit).unwrap()
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

    #[test]
    fn test_generate_branch_branch_name() {
        assert_eq!(
            super::generate_branch_branch_name("Hello & Goodbye"),
            "ps/branch/hello___goodbye"
        );
    }
}
