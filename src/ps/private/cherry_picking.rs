use super::super::super::ps;
use super::git;
use std::result::Result;

pub struct CherryPickRange {
    pub root_oid: git2::Oid,
    pub leaf_oid: Option<git2::Oid>,
}

#[derive(Debug)]
pub enum MapRangeForCherryPickError {
    StartIndexNotFound,
    EndIndexNotFound,
}

impl std::fmt::Display for MapRangeForCherryPickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartIndexNotFound => write!(f, "start index not found"),
            Self::EndIndexNotFound => write!(f, "end index not found"),
        }
    }
}

impl std::error::Error for MapRangeForCherryPickError {}

/// Convert from a `start_patch_index` and an optional `end_patch_index` into a `CherryPickRange`
/// containing a `root_oid` and optional `leaf_oid` that are ready to be used with the
/// cherry_pick() function.
///
/// Basically this maps either an individual patch index or patch series indexes from the world of
/// patch stack down to the world of Git so that the cherry_pick() operation can be performed.
///
/// The `start_patch_index` specifies either the only patch to include in the cherry pick or the
/// beginning patch of a patch series to be cherry picked. It is inclusive, meaning the patch
/// matching the index will be cherry picked.
///
/// The `end_patch_index` specifies the patch index to the patch that ends the series of patches
/// you want to cherry pick. It is inclusive, meaning the patch matching the index will be cherry
/// picked. If you want to only cherry pick the patch that matches the `start_patch_index` you can
/// simply set `end_patch_index` to `None`.
///
/// Returns a `CherryPickRange` containing a `root_oid` and `leaf_oid` ready to be used with the
/// cherry_pick() function to perform cherry pick either a single patch or a series of patches.
pub fn map_range_for_cherry_pick(
    patches_vec: &[ps::ListPatch],
    start_patch_index: usize,
    end_patch_index: Option<usize>,
) -> Result<CherryPickRange, MapRangeForCherryPickError> {
    let start_patch_oid = patches_vec
        .get(start_patch_index)
        .ok_or(MapRangeForCherryPickError::StartIndexNotFound)?
        .oid;

    Ok(match (start_patch_index, end_patch_index) {
        (si, Some(ei)) if (si < ei) => {
            let end_patch_oid = patches_vec
                .get(ei)
                .ok_or(MapRangeForCherryPickError::EndIndexNotFound)?
                .oid;
            CherryPickRange {
                root_oid: start_patch_oid,
                leaf_oid: Some(end_patch_oid),
            }
        }
        (si, Some(ei)) if (si > ei) => {
            let end_patch_oid = patches_vec
                .get(ei)
                .ok_or(MapRangeForCherryPickError::EndIndexNotFound)?
                .oid;
            CherryPickRange {
                root_oid: end_patch_oid,
                leaf_oid: Some(start_patch_oid),
            }
        }
        _ => CherryPickRange {
            root_oid: start_patch_oid,
            leaf_oid: None,
        },
    })
}

#[derive(Debug)]
pub enum CherryPickError {
    MergeCommitDetected(String),
    DestinationRefTargetNotFound,
    ConflictsExist(String, String),
    UnhandledError(Box<dyn std::error::Error>),
}

impl From<git2::Error> for CherryPickError {
    fn from(e: git2::Error) -> Self {
        Self::UnhandledError(e.into())
    }
}

impl std::fmt::Display for CherryPickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MergeCommitDetected(oid) => write!(f, "Merge commit detected with sha {}", oid),
            Self::DestinationRefTargetNotFound => {
                write!(f, "Destination ref couldn't find targ sha")
            }
            Self::ConflictsExist(oid_a, oid_b) => {
                write!(f, "Conflicts exist between shas {} and {}", oid_a, oid_b)
            }
            Self::UnhandledError(boxed_e) => write!(f, "{}", boxed_e),
        }
    }
}

impl std::error::Error for CherryPickError {}

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
/// The `committer_time_offset` specifies how much to offset the commiter time by in seconds.
///
/// The `add_missing_patch_ids` boolean specifies if it should add patch ids to commits missing
/// them that are involved in the cherry pick.
///
/// The `root_inclusive` boolean specifies if it should treat the root_oid as inclusive/exclusive
/// in the cherry pick.
///
/// It returns an Ok(Option(last_cherry_picked_commit_oid)) result in the case of success and an
/// error result of GitError in the case of failure.
pub fn cherry_pick(
    repo: &'_ git2::Repository,
    config: &git2::Config,
    root_oid: git2::Oid,
    leaf_oid: Option<git2::Oid>,
    dest_ref_name: &str,
    committer_time_offset: i64,
    add_missing_patch_ids: bool,
    root_inclusive: bool,
) -> Result<Option<git2::Oid>, CherryPickError> {
    Ok(match leaf_oid {
        Some(leaf_oid) => {
            if root_inclusive {
                let root_commit = repo.find_commit(root_oid)?;
                let root_commit_parent_commit = root_commit.parent(0)?;
                let root_commit_parent_commit_oid = root_commit_parent_commit.id();
                cherry_pick_no_working_copy_range(
                    repo,
                    config,
                    root_commit_parent_commit_oid,
                    leaf_oid,
                    dest_ref_name,
                    committer_time_offset,
                    add_missing_patch_ids,
                )?
            } else {
                cherry_pick_no_working_copy_range(
                    repo,
                    config,
                    root_oid,
                    leaf_oid,
                    dest_ref_name,
                    committer_time_offset,
                    add_missing_patch_ids,
                )?
            }
        }
        None => Some(cherry_pick_no_working_copy(
            repo,
            config,
            root_oid,
            dest_ref_name,
            committer_time_offset,
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
fn cherry_pick_no_working_copy_range(
    repo: &'_ git2::Repository,
    config: &git2::Config,
    root_oid: git2::Oid,
    leaf_oid: git2::Oid,
    dest_ref_name: &str,
    committer_time_offset: i64,
    add_missing_patch_ids: bool,
) -> Result<Option<git2::Oid>, CherryPickError> {
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
fn cherry_pick_no_working_copy<'a>(
    repo: &'a git2::Repository,
    config: &'a git2::Config,
    oid: git2::Oid,
    dest_ref_name: &str,
    committer_time_offset: i64,
    add_missing_patch_id: bool,
) -> Result<git2::Oid, CherryPickError> {
    // https://www.pygit2.org/recipes/git-cherry-pick.html#cherry-picking-a-commit-without-a-working-copy
    let commit = repo.find_commit(oid)?;
    let commit_tree = commit.tree()?;

    if commit.parents().count() > 1 {
        return Err(CherryPickError::MergeCommitDetected(
            commit.id().to_string(),
        ));
    }

    let commit_parent = commit.parent(0)?;
    let commit_parent_tree = commit_parent.tree()?;

    let destination_ref = repo.find_reference(dest_ref_name)?;
    let destination_oid = destination_ref
        .target()
        .ok_or(CherryPickError::DestinationRefTargetNotFound)?;

    let destination_commit = repo.find_commit(destination_oid)?;
    let destination_tree = destination_commit.tree()?;

    let mut index = repo.merge_trees(&commit_parent_tree, &destination_tree, &commit_tree, None)?;

    if index.has_conflicts() {
        return Err(CherryPickError::ConflictsExist(
            commit.id().to_string(),
            destination_oid.to_string(),
        ));
    }

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
        true => match ps::commit_ps_id(&commit) {
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
