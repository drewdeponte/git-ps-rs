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
  NotFound
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

/// Attempt to open an already-existing repository at or above current working
/// directory
pub fn create_cwd_repo() -> Result<git2::Repository, GitError> {
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

pub fn cherry_pick_range<'a>(repo: &'a git2::Repository, start: git2::Oid, end: git2::Oid) -> Result<(), GitError> {
  let mut rev_walk = repo.revwalk()?;
  rev_walk.push(start)?;
  rev_walk.hide(end)?;

  rev_walk.into_iter().for_each(|rev| {
    let r = rev.unwrap();
    if let Ok(commit) = repo.find_commit(r) {
      println!("- cherry-picking {}", commit.id());
      if let Ok(_) = repo.cherrypick(&commit, None) {
        println!("successfully cherry picked {}", commit.id());
      } else {
        println!("failed to cherry picked {}", commit.id());
      }
    } else {
      println!("can't find commit to cherry-pick");
    }
  });

  return Ok(());
}

pub fn cherry_pick<'a>(repo: &'a git2::Repository, oid: git2::Oid) -> Result<(), GitError> {
  if let Ok(commit) = repo.find_commit(oid) {
    println!("- cherry-picking {}", commit.id());
    if let Ok(_) = repo.cherrypick(&commit, None) {
      println!("successfully cherry picked {}", commit.id());
    } else {
      println!("failed to cherry picked {}", commit.id());
    }
  } else {
    println!("can't find commit to cherry-pick");
  }
  return Ok(());
}

// https://www.pygit2.org/recipes/git-cherry-pick.html#cherry-picking-a-commit-without-a-working-copy
pub fn cherry_pick_no_working_copy<'a>(repo: &'a git2::Repository, oid: git2::Oid, destination: git2::Branch) -> Result<(), GitError> {
  let commit = repo.find_commit(oid).unwrap();
  let commit_tree = commit.tree().unwrap();

  let destination_ref = destination.get();
  let destination_oid = destination_ref.target().unwrap();

  let common_ancestor_oid = repo.merge_base(oid, destination_oid).unwrap();
  let common_ancestor_commit = repo.find_commit(common_ancestor_oid).unwrap();
  let common_ancestor_tree = common_ancestor_commit.tree().unwrap();

  let destination_commit = repo.find_commit(destination_oid).unwrap();
  let destination_tree = destination_commit.tree().unwrap();

  let mut index = repo.merge_trees(&common_ancestor_tree, &destination_tree, &commit_tree, None).unwrap();
  let tree_oid = index.write_tree_to(repo).unwrap();
  let tree = repo.find_tree(tree_oid).unwrap();

  let destination_ref_name = destination_ref.name().unwrap();

  let author = commit.author();
  let committer = commit.committer();
  let message = commit.message().unwrap();

  let new_commit_oid = repo.commit(Option::Some(destination_ref_name), &author, &committer, message, &tree, &[&destination_commit]).unwrap();

  // repo.commit(Option::Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit]).unwrap()



  // let sig = git2::Signature::now("Bob Villa", "bob@example.com").unwrap();

  // // create the blob record for storing the content
  // let blob_oid = repo.blob(data).unwrap();
  // // repo.find_blob(blob_oid).unwrap();

  // // create the tree record
  // let mut treebuilder = repo.treebuilder(Option::None).unwrap();
  // let file_mode: i32 = i32::from(git2::FileMode::Blob);
  // treebuilder.insert(path, blob_oid, file_mode).unwrap();
  // let tree_oid = treebuilder.write().unwrap();

  // // lookup the tree entity
  // let tree = repo.find_tree(tree_oid).unwrap();

  // // TODO: need to figure out some way to get the parent commit as a
  // // git2::Commit object to hand
  // // into the repo.commit call. I am guessing that is why I am getting
  // // the following error
  // // "failed to create commit: current tip is not the first parent"
  // let parent_oid = repo.head().unwrap().target().unwrap();
  // let parent_commit = repo.find_commit(parent_oid).unwrap();

  // // create the actual commit packaging the blob, tree entry, etc.
  // repo.commit(Option::Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit]).unwrap()

  // if let Ok(commit) = repo.find_commit(oid) {
  //   println!("- cherry-picking {}", commit.id());
  //   if let Ok(_) = repo.cherrypick(&commit, None) {
  //     println!("successfully cherry picked {}", commit.id());
  //   } else {
  //     println!("failed to cherry picked {}", commit.id());
  //   }
  // } else {
  //   println!("can't find commit to cherry-pick");
  // }
  return Ok(());
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
