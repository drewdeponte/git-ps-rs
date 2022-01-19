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
    Ok(head_ref) => return head_ref.shorthand().map(String::from), 
    Err(_) => return None
  }
}

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
