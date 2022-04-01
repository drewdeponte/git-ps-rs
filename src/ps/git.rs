// This is the `git` module. It is responsible for housing
// functionality for interacting with git. Nothing in here should explicitly
// introduce patch stack concepts but obviously should be needed to support
// implementing the Patch Stack solutions at a higher level.
//
// Lets look at an example to make this more clear.
//
// fn get_commits(ps: PatchStack) -> Vec<Commit> // bad example
//
// The above is something that should NOT live in here because it introduces a
// concept specific to Patch Stack, in this case the `PatchStack` struct.
//
// We can still have the same functionality in here as it is mostly specific
// to git. If we simply write the function at the conceptual level of git
// instead it might look something like the following.
//
// fn get_comimts(head: Oid, base: Oid) -> Vec<Commit> // good example
//
// In the above two examples we can see that we are effectively providing
// the same functionality the but the API we are exposing at this level is
// constrained to the conceptual level of git and isn't aware of any Patch
// Stack specific concepts.
//
// This explicitly intended to NOT wrap libgit2. Instead it is designed to
// extend the functionality of libgit2. This means that it's functions will
// consume libgit2 types as well as potentially return libgit2 types.
//
// All code fitting that description belongs here.

use git2;

#[derive(Debug)]
pub enum GitError {
  GitError(git2::Error),
  NotFound,
  TargetNotFound,
  ReferenceNameMissing,
  CommitMessageMissing
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

#[derive(Debug)]
pub enum CreateCwdRepositoryError {
  Failed(git2::Error)
}

impl From<git2::Error> for CreateCwdRepositoryError {
  fn from(e: git2::Error) -> Self {
    Self::Failed(e)
  }
}

/// Attempt to open an already-existing repository at or above current working
/// directory
pub fn create_cwd_repo() -> Result<git2::Repository, CreateCwdRepositoryError> {
    let repo = git2::Repository::discover("./")?;
    Ok(repo)
}

/// Get Commit Summary given a repository & oid
pub fn get_summary(repo: &git2::Repository, oid: &git2::Oid) -> Result<String, GitError>{
    Ok(String::from(repo.find_commit(*oid)?
                        .summary().ok_or(GitError::NotFound)?))
}

/// Attempt to get uptream branch name given local branch name
pub fn branch_upstream_name(repo: &git2::Repository, branch_name: &str) -> Result<String, GitError> {
  let upstream_branch_name_buf = repo.branch_upstream_name(branch_name)?;
  Ok(String::from(upstream_branch_name_buf.as_str().ok_or(GitError::NotFound)?))
}

/// Attempt to get revs given a repo, start Oid (excluded), and end Oid (included)
pub fn get_revs<'a>(repo: &'a git2::Repository, start: git2::Oid, end: git2::Oid) -> Result<git2::Revwalk<'a>, GitError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push(end)?;
    rev_walk.hide(start)?;
    rev_walk.set_sorting(git2::Sort::REVERSE)?;
    Ok(rev_walk)
}

pub fn get_current_branch<'a>(repo: &'a git2::Repository) -> Option<String> {
  // https://stackoverflow.com/questions/12132862/how-do-i-get-the-name-of-the-current-branch-in-libgit2
  match repo.head() {
    Ok(head_ref) => return head_ref.name().map(String::from),
    Err(_) => return None
  }
}

// Cherry pick the range of commits onto the destination.
//
// Note: The range of commits is bounded by the `start` and `end` Oids. It is
// important to recoginze that commits referenced by `start` and `end` are
// both excluded from the actually cherry picking as they just define the
// commits surrounding the range of the commits to actually cherry-pick. It is
// also important to understand that `start` should be the most recent commit
// in history and the `end` should be the least recent commit in the range.
// Think you are starting at the top of the tree going down.
pub fn cherry_pick_no_working_copy_range<'a>(repo: &'a git2::Repository, start: git2::Oid, end: git2::Oid, dest_ref_name: &str) -> Result<(), GitError> {
  let mut rev_walk = repo.revwalk()?;
  rev_walk.set_sorting(git2::Sort::REVERSE)?;
  rev_walk.push(start)?;
  rev_walk.hide(end)?;

  for rev in rev_walk {
    if let Ok(r) = rev {
      if r == start {
        return Ok(()); // effectively short-circuit doing nothing for this last patch
      }

      repo
        .find_commit(r)
        .map_err(|e| GitError::GitError(e))
        .and_then(|commit| cherry_pick_no_working_copy(repo, commit.id(), dest_ref_name))?;
    }
  }

  return Ok(());
}

pub fn cherry_pick_no_working_copy<'a>(repo: &'a git2::Repository, oid: git2::Oid, dest_ref_name: &str) -> Result<git2::Oid, GitError> {
  // https://www.pygit2.org/recipes/git-cherry-pick.html#cherry-picking-a-commit-without-a-working-copy
  let commit = repo.find_commit(oid)?;
  let commit_tree = commit.tree()?;

  let commit_parent = commit.parent(0)?;
  let commit_parent_tree = commit_parent.tree()?;

  let destination_ref = repo.find_reference(dest_ref_name)?;
  let destination_oid = destination_ref.target().ok_or(GitError::TargetNotFound)?;

  // let common_ancestor_oid = repo.merge_base(oid, destination_oid)?;
  // let common_ancestor_commit = repo.find_commit(common_ancestor_oid)?;
  // let common_ancestor_tree = common_ancestor_commit.tree()?;

  let destination_commit = repo.find_commit(destination_oid)?;
  let destination_tree = destination_commit.tree()?;

  let mut index = repo.merge_trees(&commit_parent_tree, &destination_tree, &commit_tree, None)?;
  let tree_oid = index.write_tree_to(repo)?;
  let tree = repo.find_tree(tree_oid)?;

  let destination_ref_name = destination_ref.name().ok_or(GitError::ReferenceNameMissing)?;

  let author = commit.author();
  let committer = commit.committer();
  let message = commit.message().unwrap();

  let new_commit_oid = repo.commit(Option::Some(destination_ref_name), &author, &committer, message, &tree, &[&destination_commit])?;

  return Ok(new_commit_oid);
}

pub fn cherry_pick_no_working_copy_amend_message<'a>(repo: &'a git2::Repository, oid: git2::Oid, dest_ref_name: &str, message_amendment: &str) -> Result<git2::Oid, GitError> {
  // https://www.pygit2.org/recipes/git-cherry-pick.html#cherry-picking-a-commit-without-a-working-copy
  let commit = repo.find_commit(oid)?;
  let commit_tree = commit.tree()?;

  let commit_parent = commit.parent(0)?;
  let commit_parent_tree = commit_parent.tree()?;

  let destination_ref = repo.find_reference(dest_ref_name)?;
  let destination_oid = destination_ref.target().ok_or(GitError::TargetNotFound)?;

  // let common_ancestor_oid = repo.merge_base(oid, destination_oid)?;
  // let common_ancestor_commit = repo.find_commit(common_ancestor_oid)?;
  // let common_ancestor_tree = common_ancestor_commit.tree()?;

  let destination_commit = repo.find_commit(destination_oid)?;
  let destination_tree = destination_commit.tree()?;

  let mut index = repo.merge_trees(&commit_parent_tree, &destination_tree, &commit_tree, None)?;
  let tree_oid = index.write_tree_to(repo)?;
  let tree = repo.find_tree(tree_oid)?;

  let destination_ref_name = destination_ref.name().ok_or(GitError::ReferenceNameMissing)?;

  let author = commit.author();
  let committer = commit.committer();
  let message = commit.message().ok_or(GitError::CommitMessageMissing)?;
  let amended_message = format!("{}{}", message, message_amendment);

  let new_commit_oid = repo.commit(Option::Some(destination_ref_name), &author, &committer, amended_message.as_str(), &tree, &[&destination_commit])?;

  return Ok(new_commit_oid);
}

// private func addIdTo(uuid: UUID, patch: Commit) throws -> Commit? {
//   let originalBranch = try self.git.getCheckedOutBranch()
//   let upstreamBranch = try self.git.getUpstreamBranch()
//   let commonAncestorRef = try self.git.mergeBase(refA: patch.hash, refB: upstreamBranch.remoteBase)
//   try self.git.createAndCheckout(branch: "ps/tmp/add_id_rework", startingFrom: commonAncestorRef)
//   try self.git.cherryPickCommits(from: commonAncestorRef, to: patch.hash)
//   let shaOfPatchPrime = try self.git.getShaOf(ref: "HEAD")
//   print("- got sha of HEAD (a.k.a. patch') - \(shaOfPatchPrime)")
//   let originalMessage = try self.git.commitMessageOf(ref: shaOfPatchPrime)
//   print("- got commit message from \(shaOfPatchPrime) (a.k.a. patch')")
//   try self.git.commitAmendMessages(messages: [originalMessage, "ps-id: \(uuid.uuidString)"])
//   print("- amended patch' wich ps-id: \(uuid.uuidString), it is now patch''")
//   let shaOfPatchFinalPrime = try self.git.getShaOf(ref: "HEAD")
//   print("- got sha of HEAD (a.k.a. patch'' - \(shaOfPatchFinalPrime)")
//   try self.git.cherryPickCommits(from: patch.hash, to: upstreamBranch.branch)
//   try self.git.forceBranch(named: upstreamBranch.branch, to: "HEAD")
//   print("- forced branch (\(upstreamBranch.branch)) to point to HEAD")
//   try self.git.checkout(ref: originalBranch)
//   print("- checked out branch - \(originalBranch)")
//   try self.git.deleteBranch(named: "ps/tmp/add_id_rework")
//   print("- deleted tmp working branch - ps/tmp/add_id_rework")
//   return try self.git.commit(shaOfPatchFinalPrime)
// }

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_get_summary() {
        let (_td, repo) = crate::ps::test::repo_init();
        let head_id = repo.refname_to_id("HEAD").unwrap();

        let res = super::get_summary(&repo, &head_id);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "initial");
    }

    #[test]
    fn smoke_get_revs() {
        let (_td, repo) = crate::ps::test::repo_init();

        let start_oid_excluded = crate::ps::test::create_commit(&repo, "fileA.txt", &[0, 1, 2, 3], "starting numbers");
        crate::ps::test::create_commit(&repo, "fileB.txt", &[4, 5, 6, 7], "four, five, six, and seven");
        let end_oid_included = crate::ps::test::create_commit(&repo, "fileC.txt", &[8, 9, 10, 11], "eight, nine, ten, and eleven");
        crate::ps::test::create_commit(&repo, "fileD.txt", &[12, 13, 14, 15], "twelve, thirteen, forteen, fifteen");

        let rev_walk = super::get_revs(&repo, start_oid_excluded, end_oid_included).unwrap();
        let summaries: Vec<String> = rev_walk.map(|oid| repo.find_commit(oid.unwrap()).unwrap().summary().unwrap().to_string()).collect();
        assert_eq!(summaries.len(), 2);

        assert_eq!(summaries.first().unwrap(), "four, five, six, and seven");
        assert_eq!(summaries.last().unwrap(), "eight, nine, ten, and eleven");
    }
}
