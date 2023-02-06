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
use gpgme;
use super::utils;
use std::result::Result;
use std::str;

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
pub fn get_revs(repo: &git2::Repository, start: git2::Oid, end: git2::Oid, sort: git2::Sort) -> Result<git2::Revwalk, GitError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push(end)?;
    rev_walk.hide(start)?;
    rev_walk.set_sorting(sort)?;
    Ok(rev_walk)
}

pub fn get_current_branch(repo: &git2::Repository) -> Option<String> {
  // https://stackoverflow.com/questions/12132862/how-do-i-get-the-name-of-the-current-branch-in-libgit2
  match repo.head() {
    Ok(head_ref) => return head_ref.name().map(String::from),
    Err(_) => None
  }
}

pub fn get_current_branch_shorthand(repo: &git2::Repository) -> Option<String> {
  // https://stackoverflow.com/questions/12132862/how-do-i-get-the-name-of-the-current-branch-in-libgit2
  match repo.head() {
    Ok(head_ref) => return head_ref.shorthand().map(String::from),
    Err(_) => None
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
pub fn cherry_pick_no_working_copy_range<'a>(repo: &'a git2::Repository, config: &git2::Config, start: git2::Oid, end: git2::Oid, dest_ref_name: &str) -> Result<(), GitError> {
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
        .and_then(|commit| cherry_pick_no_working_copy(repo, config, commit.id(), dest_ref_name, 0))?;
    }
  }

  return Ok(());
}

#[derive(Debug)]
pub enum ConfigGetError {
  Failed(git2::Error)
}

pub fn config_get_to_option<T>(res_val: Result<T, git2::Error>) -> Result<Option<T>, ConfigGetError> {
  match res_val {
    Ok(v) => Ok(Some(v)),
    Err(e) => {
      if e.class() == git2::ErrorClass::Config && e.code() == git2::ErrorCode::NotFound {
        Ok(None)
      } else {
        Err(ConfigGetError::Failed(e))
      }
    }
  }
}

pub fn config_get_bool(config: &git2::Config, name: &str) -> Result<Option<bool>, ConfigGetError> {
  config_get_to_option(config.get_bool(name))
}

pub fn config_get_string(config: &git2::Config, name: &str) -> Result<Option<String>, ConfigGetError> {
  config_get_to_option(config.get_string(name))
}

#[derive(Debug)]
pub enum CreateCommitError {
  GetCommitGpgsignFailed(ConfigGetError),
  GetGpgFormatFailed(ConfigGetError),
  GetUserSigningKeyFailed(ConfigGetError),
  CreateGpgSignedCommitFailed(CreateGpgSignedCommitError),
  CreateUnsignedCommitFailed(CreateUnsignedCommitError),
  UserSigningKeyNotFoundInGitConfig
}

pub fn create_commit(
  repo: &'_ git2::Repository,
  config: &'_ git2::Config,
  dest_ref_name: &str,
  author: &git2::Signature<'_>,
  committer: &git2::Signature<'_>,
  message: &str,
  tree: &git2::Tree<'_>,
  parents: &[&git2::Commit<'_>]
) -> Result<git2::Oid, CreateCommitError>{

  // let config = git2::Config::open_default().unwrap();
  // let sign_commit_flag_result = config.get_bool("commit.gpgsign");

  let sign_commit_flag = config_get_bool(config, "commit.gpgsign")
    .map_err(CreateCommitError::GetCommitGpgsignFailed)?
    .unwrap_or(false);

  if sign_commit_flag {
    let gpg_format_option = config_get_string(config, "gpg.format")
      .map_err(CreateCommitError::GetGpgFormatFailed)?;
    let sign_with_gpg = match gpg_format_option {
      Some(v) => v == "openpgp",
      None => true
    };

    if sign_with_gpg {
      let signing_key = config_get_string(config, "user.signingkey")
        .map_err(CreateCommitError::GetUserSigningKeyFailed)?
        .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;
      create_gpg_signed_commit(repo, signing_key, dest_ref_name, author, committer, message, tree, parents)
        .map_err(CreateCommitError::CreateGpgSignedCommitFailed)
    } else {
      eprintln!("Warning: gps currently only supports GPG signatures. See issues #44 & #45 - https://github.com/uptech/git-ps-rs/issues");
      eprintln!("The commits have been created unsigned!");
      create_unsigned_commit(repo, dest_ref_name, author, committer, message, tree, parents)
        .map_err(CreateCommitError::CreateUnsignedCommitFailed)
    }
  } else {
    create_unsigned_commit(repo, dest_ref_name, author, committer, message, tree, parents)
      .map_err(CreateCommitError::CreateUnsignedCommitFailed)
  }
}

#[derive(Debug)]
pub enum CreateGpgSignedCommitError {
  CreateCommitBufferFailed(git2::Error),
  FromUtf8Failed(str::Utf8Error),
  GpgSignStringFailed(GpgSignStringError),
  FindDestinationReferenceFailed(git2::Error),
  CommitSignedFailed(git2::Error),
  SetReferenceTargetFailed(git2::Error)
}

pub fn create_gpg_signed_commit(
  repo: &'_ git2::Repository,
  signing_key: String,
  dest_ref_name: &str,
  author: &git2::Signature<'_>,
  committer: &git2::Signature<'_>,
  message: &str,
  tree: &git2::Tree<'_>,
  parents: &[&git2::Commit<'_>]
) -> Result<git2::Oid, CreateGpgSignedCommitError> {

  // create commit buffer as a string so that we can sign it
  let commit_buf = repo.commit_create_buffer(author, committer, message, tree, parents)
    .map_err(CreateGpgSignedCommitError::CreateCommitBufferFailed)?;
  let commit_as_str = str::from_utf8(&commit_buf)
    .map_err(CreateGpgSignedCommitError::FromUtf8Failed)?
    .to_string();

  // create digital signature from commit buf
  let sig = gpg_sign_string(commit_as_str.clone(), signing_key)
    .map_err(CreateGpgSignedCommitError::GpgSignStringFailed)?;

  // lookup the given reference
  let mut destination_ref = repo.find_reference(dest_ref_name)
    .map_err(CreateGpgSignedCommitError::FindDestinationReferenceFailed)?;

  let new_commit_oid = repo.commit_signed(&commit_as_str, &sig, Some("gpgsig"))
    .map_err(CreateGpgSignedCommitError::CommitSignedFailed)?;

  // set the ref target
  destination_ref.set_target(new_commit_oid, "create commit signed commit")
    .map_err(CreateGpgSignedCommitError::SetReferenceTargetFailed)?;

  Ok(new_commit_oid)
}

#[derive(Debug)]
pub enum GpgSignStringError {
  GetGpgContextFailed,
  GetSecretKeyFailed,
  AddSignerFailed,
  CreateDetachedSignatureFailed,
  FromUtf8Failed(std::string::FromUtf8Error)
}

pub fn gpg_sign_string(commit: String, signing_key: String) -> Result<String, GpgSignStringError> {
    let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp).map_err(|_| GpgSignStringError::GetGpgContextFailed)?;
    ctx.set_armor(true);
    let key = ctx.get_secret_key(signing_key).map_err(|_| GpgSignStringError::GetSecretKeyFailed)?;

    ctx.add_signer(&key).map_err(|_| GpgSignStringError::AddSignerFailed)?;
    let mut output = Vec::new();
    ctx.sign_detached(commit, &mut output).map_err(|_| GpgSignStringError::CreateDetachedSignatureFailed)?;

    String::from_utf8(output).map(|s| s.trim().to_string()).map_err(GpgSignStringError::FromUtf8Failed)
}

#[derive(Debug)]
pub enum CreateUnsignedCommitError {
  FindDestinationReferenceFailed(git2::Error),
  DestinationReferenceNameNotFound,
  CreateCommitFailed(git2::Error)
}

pub fn create_unsigned_commit(
  repo: &'_ git2::Repository,
  dest_ref_name: &str,
  author: &git2::Signature<'_>,
  committer: &git2::Signature<'_>,
  message: &str,
  tree: &git2::Tree<'_>,
  parents: &[&git2::Commit<'_>]
) -> Result<git2::Oid, CreateUnsignedCommitError> {
  let destination_ref = repo.find_reference(dest_ref_name)
    .map_err(CreateUnsignedCommitError::FindDestinationReferenceFailed)?;
  let destination_ref_name = destination_ref.name()
    .ok_or(CreateUnsignedCommitError::DestinationReferenceNameNotFound)?;
  let new_commit_oid = repo.commit(Option::Some(destination_ref_name), author, committer, message, tree, parents)
    .map_err(CreateUnsignedCommitError::CreateCommitFailed)?;
  Ok(new_commit_oid)
}

/// Cherry pick the commit identified by the oid to the dest_ref_name with the
/// given committer_time_offset. Note: The committer_time_offset is used to
/// offset the Commiter's signature timestamp which is in seconds since epoch
/// so that if we are performing multiple operations on the same commit within
/// less than a second we can offset it in one direction or the other. The
/// current use case for this is when we add patch stack id to a commit and
/// then immediately cherry pick that commit into the ps/rr/whatever branch as
/// part of the request_review_branch() operation.
pub fn cherry_pick_no_working_copy<'a>(repo: &'a git2::Repository, config: &'a git2::Config, oid: git2::Oid, dest_ref_name: &str, committer_time_offset: i64) -> Result<git2::Oid, GitError> {
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

  let author = commit.author();
  let committer = repo.signature().unwrap();

  let message = commit.message().unwrap();

  let new_time = git2::Time::new(committer.when().seconds() + committer_time_offset, committer.when().offset_minutes());
  let new_committer = git2::Signature::new(committer.name().unwrap(), committer.email().unwrap(), &new_time).unwrap();

  let new_commit_oid = create_commit(repo, config, dest_ref_name, &author, &new_committer, message, &tree, &[&destination_commit]).unwrap();

  Ok(new_commit_oid)
}

pub fn cherry_pick_no_working_copy_amend_message<'a>(repo: &'a git2::Repository, config: &git2::Config, oid: git2::Oid, dest_ref_name: &str, message_amendment: &str) -> Result<git2::Oid, GitError> {
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

  let author = commit.author();
  let committer = repo.signature().unwrap();
  let message = commit.message().ok_or(GitError::CommitMessageMissing)?;
  let amended_message = format!("{}{}", message, message_amendment);

  // let new_commit_oid = repo.commit(Option::Some(destination_ref_name), &author, &committer, amended_message.as_str(), &tree, &[&destination_commit])?;
  let new_commit_oid = create_commit(repo, config, dest_ref_name, &author, &committer, amended_message.as_str(), &tree, &[&destination_commit]).unwrap();

  Ok(new_commit_oid)
}

#[derive(Debug)]
pub enum ExtForcePushError {
  ExecuteFailed(utils::ExecuteError)
}

pub fn ext_push(force: bool, remote_name: &str, src_ref_spec: &str, dest_ref_spec: &str) -> Result<(), ExtForcePushError> {
  let refspecs = format!("{}:{}", src_ref_spec, dest_ref_spec);
  if force {
    utils::execute("git", &["push", "-f", remote_name, &refspecs]).map_err(|e| ExtForcePushError::ExecuteFailed(e))
  } else {
    utils::execute("git", &["push", remote_name, &refspecs]).map_err(|e| ExtForcePushError::ExecuteFailed(e))
  }
}

#[derive(Debug)]
pub enum ExtDeleteRemoteBranchError {
  ExecuteFailed(utils::ExecuteError)
}

pub fn ext_delete_remote_branch(remote_name: &str, branch_name: &str) -> Result<(), ExtDeleteRemoteBranchError> {
  let refspecs = format!(":{}", branch_name);
  utils::execute("git", &["push", remote_name, &refspecs]).map_err(ExtDeleteRemoteBranchError::ExecuteFailed)?;
  Ok(())
}

#[derive(Debug)]
pub enum ExtFetchError {
  ExecuteFailed(utils::ExecuteError)
}

pub fn ext_fetch() -> Result<(), ExtFetchError> {
  utils::execute("git", &["fetch"]).map_err(ExtFetchError::ExecuteFailed)?;
  Ok(())
}

#[derive(Debug)]
pub enum CommitDiffError {
  MergeCommit,
  CommitParentCountZero,
  GetParentZeroFailed,
  GetParentZeroCommitFailed,
  GetParentZeroTreeFailed,
  GetCommitTreeFailed,
  GetDiffTreeToTreeFailed
}

pub fn commit_diff<'a>(repo: &'a git2::Repository, commit: &git2::Commit) -> Result<git2::Diff<'a>, CommitDiffError> {
  if commit.parent_count() > 1 {
    return Err(CommitDiffError::MergeCommit)
  }

  if commit.parent_count() > 0 {
    let parent_oid = commit.parent_id(0).map_err(|_| CommitDiffError::GetParentZeroFailed)?;
    let parent_commit = repo.find_commit(parent_oid).map_err(|_| CommitDiffError::GetParentZeroCommitFailed)?;
    let parent_tree = parent_commit.tree().map_err(|_| CommitDiffError::GetParentZeroTreeFailed)?;

    let commit_tree = commit.tree().map_err(|_| CommitDiffError::GetCommitTreeFailed)?;
    Ok(repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Option::None).map_err(|_| CommitDiffError::GetDiffTreeToTreeFailed)?)
  } else {
    Err(CommitDiffError::CommitParentCountZero)
  }
}

#[derive(Debug)]
pub enum CommitDiffPatchIdError {
  GetDiffFailed(CommitDiffError),
  CreatePatchHashFailed(git2::Error)
}

pub fn commit_diff_patch_id(repo: &git2::Repository, commit: &git2::Commit) -> Result<git2::Oid, CommitDiffPatchIdError> {
  let diff = commit_diff(repo, commit).map_err(CommitDiffPatchIdError::GetDiffFailed)?;
  diff.patchid(Option::None).map_err(CommitDiffPatchIdError::CreatePatchHashFailed)
}

#[derive(Debug)]
pub enum CommonAncestorError {
  MergeBaseFailed(git2::Error),
  FindCommitFailed(git2::Error),
  GetParentZeroFailed(git2::Error)
}

pub fn common_ancestor(repo: &git2::Repository, one: git2::Oid, two: git2::Oid) -> Result<git2::Oid, CommonAncestorError> {
  let merge_base_oid = repo.merge_base(one, two).map_err(CommonAncestorError::MergeBaseFailed)?;
  Ok(merge_base_oid)
}

#[derive(Debug)]
pub enum SingularCommitOfBranchError {
  CommonAncestorFailed(CommonAncestorError),
  FindBranchFailed(git2::Error),
  BranchMissingTarget,
  GetRevWalkerFailed(GitError),
  BranchDoesntHaveExactlyOneCommit(String, usize),
  FindBranchCommitFailed(git2::Error)
}

pub fn singular_commit_of_branch<'a>(repo: &'a git2::Repository, branch_name: &str, branch_type: git2::BranchType, base_oid: git2::Oid) -> Result<git2::Commit<'a>, SingularCommitOfBranchError> {
  let branch_oid = repo.find_branch(branch_name, branch_type).map_err(SingularCommitOfBranchError::FindBranchFailed)?.get().target().ok_or(SingularCommitOfBranchError::BranchMissingTarget)?;
  let common_ancestor_oid = common_ancestor(repo, branch_oid, base_oid).map_err(SingularCommitOfBranchError::CommonAncestorFailed)?;

  let revwalk = get_revs(repo, common_ancestor_oid, branch_oid, git2::Sort::REVERSE).map_err(SingularCommitOfBranchError::GetRevWalkerFailed)?;
  let num_of_commits = revwalk.count();

  if num_of_commits > 1 || num_of_commits == 0 && common_ancestor_oid != branch_oid {
    Err(SingularCommitOfBranchError::BranchDoesntHaveExactlyOneCommit(branch_name.to_string(), num_of_commits))
  } else {
    repo.find_commit(branch_oid).map_err(SingularCommitOfBranchError::FindBranchCommitFailed)
  }
}

#[derive(Debug)]
pub enum UncommittedChangesError {
  StatusesFailed(git2::Error)
}

pub fn uncommitted_changes_exist(repo: &git2::Repository) -> Result<bool, UncommittedChangesError> {
  let mut status_options = git2::StatusOptions::default();
  status_options.show(git2::StatusShow::Workdir);
  status_options.include_untracked(true);
  let statuses = repo.statuses(Some(&mut status_options)).map_err(UncommittedChangesError::StatusesFailed)?;
  Ok(!statuses.is_empty())
}

#[derive(Debug)]
pub enum HashObjectWriteError {
  Failed(git2::Error)
}

#[allow(dead_code)]
pub fn hash_object_write(repo: &git2::Repository, content: &str) -> Result<git2::Oid, HashObjectWriteError> {
  repo.blob(content.as_bytes()).map_err(HashObjectWriteError::Failed)
}

#[derive(Debug)]
pub enum ReadHashedObjectError {
  NotValidUtf8(std::str::Utf8Error),
  Failed(git2::Error)
}

#[allow(dead_code)]
pub fn read_hashed_object(repo: &git2::Repository, oid: git2::Oid) -> Result<String, ReadHashedObjectError> {
  let blob = repo.find_blob(oid).map_err(ReadHashedObjectError::Failed)?;
  let content = blob.content();
  let str_ref = std::str::from_utf8(content).map_err(ReadHashedObjectError::NotValidUtf8)?;
  Ok(str_ref.to_string())
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;
  use git2::{Repository, RepositoryInitOptions};

  pub fn repo_init() -> (TempDir, Repository) {
      let td = TempDir::new().unwrap();
      let mut opts = RepositoryInitOptions::new();
      opts.initial_head("main");
      let repo = Repository::init_opts(td.path(), &opts).unwrap();
      {
          let mut config = repo.config().unwrap();
          config.set_str("user.name", "name").unwrap();
          config.set_str("user.email", "email").unwrap();
          let mut index = repo.index().unwrap();

          let id = index.write_tree().unwrap();
          let tree = repo.find_tree(id).unwrap();
          let sig = repo.signature().unwrap();
          repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
              .unwrap();
      }
      (td, repo)
  }

  pub fn create_commit(repo: &git2::Repository, path: &str, data: &[u8], message: &str) -> git2::Oid {
    // To implement this I was losely following
    // https://stackoverflow.com/questions/15711444/how-to-commit-to-a-git-repository-using-libgit2
    let sig = git2::Signature::now("Bob Villa", "bob@example.com").unwrap();

    // create the blob record for storing the content
    let blob_oid = repo.blob(data).unwrap();
    // repo.find_blob(blob_oid).unwrap();

    // create the tree record
    let mut treebuilder = repo.treebuilder(Option::None).unwrap();
    let file_mode: i32 = i32::from(git2::FileMode::Blob);
    treebuilder.insert(path, blob_oid, file_mode).unwrap();
    let tree_oid = treebuilder.write().unwrap();

    // lookup the tree entity
    let tree = repo.find_tree(tree_oid).unwrap();

    // TODO: need to figure out some way to get the parent commit as a
    // git2::Commit object to hand
    // into the repo.commit call. I am guessing that is why I am getting
    // the following error
    // "failed to create commit: current tip is not the first parent"
    let parent_oid = repo.head().unwrap().target().unwrap();
    let parent_commit = repo.find_commit(parent_oid).unwrap();

    // create the actual commit packaging the blob, tree entry, etc.
    repo.commit(Option::Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit]).unwrap()
  }

    #[test]
    fn smoke_get_summary() {
        let (_td, repo) = repo_init();
        let head_id = repo.refname_to_id("HEAD").unwrap();

        let res = super::get_summary(&repo, &head_id);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "initial");
    }

    #[test]
    fn smoke_get_revs() {
        let (_td, repo) = repo_init();

        let start_oid_excluded = create_commit(&repo, "fileA.txt", &[0, 1, 2, 3], "starting numbers");
        create_commit(&repo, "fileB.txt", &[4, 5, 6, 7], "four, five, six, and seven");
        let end_oid_included = create_commit(&repo, "fileC.txt", &[8, 9, 10, 11], "eight, nine, ten, and eleven");
        create_commit(&repo, "fileD.txt", &[12, 13, 14, 15], "twelve, thirteen, forteen, fifteen");

        let rev_walk = super::get_revs(&repo, start_oid_excluded, end_oid_included, git2::Sort::REVERSE).unwrap();
        let summaries: Vec<String> = rev_walk.map(|oid| repo.find_commit(oid.unwrap()).unwrap().summary().unwrap().to_string()).collect();
        assert_eq!(summaries.len(), 2);

        assert_eq!(summaries.first().unwrap(), "four, five, six, and seven");
        assert_eq!(summaries.last().unwrap(), "eight, nine, ten, and eleven");
    }

  #[test]
  fn test_hash_object_write() {
    let (_td, repo) = repo_init();
    let message = "Hello hash object write!";
    let oid = super::hash_object_write(&repo, message).unwrap();
    let blob = repo.find_blob(oid).unwrap();
    assert_eq!(blob.content(), message.as_bytes());
  }

  #[test]
  fn test_read_hashed_object() {
    let (_td, repo) = repo_init();
    let message = "Hello hash object write!";
    let oid = super::hash_object_write(&repo, message).unwrap();
    let retreived_message = super::read_hashed_object(&repo, oid).unwrap();
    assert_eq!(retreived_message, message);
  }
}
