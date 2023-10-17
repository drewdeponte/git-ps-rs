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

use super::signers;
use super::utils;
use git2;
use std::fs;
use std::result::Result;
use std::str;

#[derive(Debug)]
pub enum GitError {
    Git(git2::Error),
    NotFound,
    TargetNotFound,
    ReferenceNameMissing,
    CommitMessageMissing,
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::Git(e)
    }
}

#[derive(Debug)]
pub enum CreateCwdRepositoryError {
    Failed(git2::Error),
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
pub fn get_summary(repo: &git2::Repository, oid: &git2::Oid) -> Result<String, GitError> {
    Ok(String::from(
        repo.find_commit(*oid)?
            .summary()
            .ok_or(GitError::NotFound)?,
    ))
}

/// Attempt to get uptream branch name given local branch name
pub fn branch_upstream_name(
    repo: &git2::Repository,
    branch_name: &str,
) -> Result<String, GitError> {
    let upstream_branch_name_buf = repo.branch_upstream_name(branch_name)?;
    Ok(String::from(
        upstream_branch_name_buf
            .as_str()
            .ok_or(GitError::NotFound)?,
    ))
}

/// Attempt to get revs given a repo, start Oid (excluded), and end Oid (included)
pub fn get_revs(
    repo: &git2::Repository,
    start: git2::Oid,
    end: git2::Oid,
    sort: git2::Sort,
) -> Result<git2::Revwalk, GitError> {
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
        Err(_) => None,
    }
}

pub fn get_current_branch_shorthand(repo: &git2::Repository) -> Option<String> {
    // https://stackoverflow.com/questions/12132862/how-do-i-get-the-name-of-the-current-branch-in-libgit2
    match repo.head() {
        Ok(head_ref) => return head_ref.shorthand().map(String::from),
        Err(_) => None,
    }
}

#[derive(Debug)]
pub enum ConfigGetError {
    Failed(git2::Error),
}

pub fn config_get_to_option<T>(
    res_val: Result<T, git2::Error>,
) -> Result<Option<T>, ConfigGetError> {
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

pub fn config_get_string(
    config: &git2::Config,
    name: &str,
) -> Result<Option<String>, ConfigGetError> {
    config_get_to_option(config.get_string(name))
}

#[derive(Debug)]
pub enum CreateCommitError {
    GetCommitGpgsignFailed(ConfigGetError),
    GetGpgFormatFailed(ConfigGetError),
    GetUserSigningKeyFailed(ConfigGetError),
    CreateSignedCommitFailed(CreateSignedCommitError),
    CreateUnsignedCommitFailed(CreateUnsignedCommitError),
    ReadSshSigningKeyFailed(std::io::Error),
    UserSigningKeyNotFoundInGitConfig,
}

pub fn create_commit(
    repo: &'_ git2::Repository,
    config: &'_ git2::Config,
    dest_ref_name: &str,
    author: &git2::Signature<'_>,
    committer: &git2::Signature<'_>,
    message: &str,
    tree: &git2::Tree<'_>,
    parents: &[&git2::Commit<'_>],
) -> Result<git2::Oid, CreateCommitError> {
    let sign_commit_flag = config_get_bool(config, "commit.gpgsign")
        .map_err(CreateCommitError::GetCommitGpgsignFailed)?
        .unwrap_or(false);

    if sign_commit_flag {
        let gpg_format_option = config_get_string(config, "gpg.format")
            .map_err(CreateCommitError::GetGpgFormatFailed)?;

        match gpg_format_option {
            Some(val) => match val.as_str() {
                "openpgp" => {
                    let signing_key = config_get_string(config, "user.signingkey")
                        .map_err(CreateCommitError::GetUserSigningKeyFailed)?
                        .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;

                    create_signed_commit(
                        repo,
                        signers::gpg_signer(signing_key),
                        dest_ref_name,
                        author,
                        committer,
                        message,
                        tree,
                        parents,
                    )
                    .map_err(CreateCommitError::CreateSignedCommitFailed)
                }
                "ssh" => {
                    let signing_key_path = config_get_string(config, "user.signingkey")
                        .map_err(CreateCommitError::GetUserSigningKeyFailed)?
                        .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;

                    let encoded_key = fs::read_to_string(&signing_key_path)
                        .map_err(CreateCommitError::ReadSshSigningKeyFailed)?;

                    create_signed_commit(
                        repo,
                        signers::ssh_signer(encoded_key, signing_key_path),
                        dest_ref_name,
                        author,
                        committer,
                        message,
                        tree,
                        parents,
                    )
                    .map_err(CreateCommitError::CreateSignedCommitFailed)
                }
                "x509" => {
                    eprintln!("Warning: gps currently does NOT support x509 signatures. See issue #44 - https://github.com/uptech/git-ps-rs/issues");
                    eprintln!("The commit has been created unsigned!");
                    create_unsigned_commit(
                        repo,
                        dest_ref_name,
                        author,
                        committer,
                        message,
                        tree,
                        parents,
                    )
                    .map_err(CreateCommitError::CreateUnsignedCommitFailed)
                }
                _ => {
                    eprintln!("Warning: gps currently only supports GPG & SSH signatures. See issue #44 - https://github.com/uptech/git-ps-rs/issues");
                    eprintln!("The commit has been created unsigned!");
                    create_unsigned_commit(
                        repo,
                        dest_ref_name,
                        author,
                        committer,
                        message,
                        tree,
                        parents,
                    )
                    .map_err(CreateCommitError::CreateUnsignedCommitFailed)
                }
            },
            None => {
                eprintln!("Warning: Your git config gpg.format doesn't appear to be set even though commit.gpgsign is enalbed");
                eprintln!("The commit has been created unsigned!");
                create_unsigned_commit(
                    repo,
                    dest_ref_name,
                    author,
                    committer,
                    message,
                    tree,
                    parents,
                )
                .map_err(CreateCommitError::CreateUnsignedCommitFailed)
            }
        }
    } else {
        create_unsigned_commit(
            repo,
            dest_ref_name,
            author,
            committer,
            message,
            tree,
            parents,
        )
        .map_err(CreateCommitError::CreateUnsignedCommitFailed)
    }
}

#[derive(Debug)]
pub enum CreateSignedCommitError {
    CreateCommitBuffer(git2::Error),
    FromUtf8(str::Utf8Error),
    SigningFailed(signers::SignerError),
    FindDestinationReference(git2::Error),
    CommitSigned(git2::Error),
    SetReferenceTarget(git2::Error),
}

pub fn create_signed_commit<F>(
    repo: &'_ git2::Repository,
    signer: F,
    dest_ref_name: &str,
    author: &git2::Signature<'_>,
    committer: &git2::Signature<'_>,
    message: &str,
    tree: &git2::Tree<'_>,
    parents: &[&git2::Commit<'_>],
) -> Result<git2::Oid, CreateSignedCommitError>
where
    F: Fn(String) -> Result<String, signers::SignerError>,
{
    // create commit buffer as a string so that we can sign it
    let commit_buf = repo
        .commit_create_buffer(author, committer, message, tree, parents)
        .map_err(CreateSignedCommitError::CreateCommitBuffer)?;
    let commit_as_str = str::from_utf8(&commit_buf)
        .map_err(CreateSignedCommitError::FromUtf8)?
        .to_string();

    let signature =
        signer(commit_as_str.clone()).map_err(CreateSignedCommitError::SigningFailed)?;

    // lookup the given reference
    let mut destination_ref = repo
        .find_reference(dest_ref_name)
        .map_err(CreateSignedCommitError::FindDestinationReference)?;

    let new_commit_oid = repo
        .commit_signed(&commit_as_str, &signature, Some("gpgsig"))
        .map_err(CreateSignedCommitError::CommitSigned)?;

    // set the ref target
    destination_ref
        .set_target(new_commit_oid, "create commit signed commit")
        .map_err(CreateSignedCommitError::SetReferenceTarget)?;

    Ok(new_commit_oid)
}

#[derive(Debug)]
pub enum CreateUnsignedCommitError {
    FindDestinationReferenceFailed(git2::Error),
    DestinationReferenceNameNotFound,
    CreateCommitFailed(git2::Error),
}

pub fn create_unsigned_commit(
    repo: &'_ git2::Repository,
    dest_ref_name: &str,
    author: &git2::Signature<'_>,
    committer: &git2::Signature<'_>,
    message: &str,
    tree: &git2::Tree<'_>,
    parents: &[&git2::Commit<'_>],
) -> Result<git2::Oid, CreateUnsignedCommitError> {
    let destination_ref = repo
        .find_reference(dest_ref_name)
        .map_err(CreateUnsignedCommitError::FindDestinationReferenceFailed)?;
    let destination_ref_name = destination_ref
        .name()
        .ok_or(CreateUnsignedCommitError::DestinationReferenceNameNotFound)?;
    let new_commit_oid = repo
        .commit(
            Option::Some(destination_ref_name),
            author,
            committer,
            message,
            tree,
            parents,
        )
        .map_err(CreateUnsignedCommitError::CreateCommitFailed)?;
    Ok(new_commit_oid)
}

#[derive(Debug)]
pub enum ExtForcePushError {
    ExecuteFailed(utils::ExecuteError),
}

pub fn ext_push(
    force: bool,
    remote_name: &str,
    src_ref_spec: &str,
    dest_ref_spec: &str,
) -> Result<(), ExtForcePushError> {
    let refspecs = format!("{}:{}", src_ref_spec, dest_ref_spec);
    if force {
        utils::execute("git", &["push", "-f", remote_name, &refspecs])
            .map_err(ExtForcePushError::ExecuteFailed)
    } else {
        utils::execute("git", &["push", remote_name, &refspecs])
            .map_err(ExtForcePushError::ExecuteFailed)
    }
}

#[derive(Debug)]
pub enum ExtDeleteRemoteBranchError {
    ExecuteFailed(utils::ExecuteError),
}

pub fn ext_delete_remote_branch(
    remote_name: &str,
    branch_name: &str,
) -> Result<(), ExtDeleteRemoteBranchError> {
    let refspecs = format!(":{}", branch_name);
    utils::execute("git", &["push", remote_name, &refspecs])
        .map_err(ExtDeleteRemoteBranchError::ExecuteFailed)?;
    Ok(())
}

#[derive(Debug)]
pub enum ExtFetchError {
    ExecuteFailed(utils::ExecuteError),
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
    GetDiffTreeToTreeFailed,
}

pub fn commit_diff<'a>(
    repo: &'a git2::Repository,
    commit: &git2::Commit,
) -> Result<git2::Diff<'a>, CommitDiffError> {
    if commit.parent_count() > 1 {
        return Err(CommitDiffError::MergeCommit);
    }

    if commit.parent_count() > 0 {
        let parent_oid = commit
            .parent_id(0)
            .map_err(|_| CommitDiffError::GetParentZeroFailed)?;
        let parent_commit = repo
            .find_commit(parent_oid)
            .map_err(|_| CommitDiffError::GetParentZeroCommitFailed)?;
        let parent_tree = parent_commit
            .tree()
            .map_err(|_| CommitDiffError::GetParentZeroTreeFailed)?;

        let commit_tree = commit
            .tree()
            .map_err(|_| CommitDiffError::GetCommitTreeFailed)?;
        Ok(repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Option::None)
            .map_err(|_| CommitDiffError::GetDiffTreeToTreeFailed)?)
    } else {
        Err(CommitDiffError::CommitParentCountZero)
    }
}

#[derive(Debug)]
pub enum CommitDiffPatchIdError {
    GetDiffFailed(CommitDiffError),
    CreatePatchHashFailed(git2::Error),
}

pub fn commit_diff_patch_id(
    repo: &git2::Repository,
    commit: &git2::Commit,
) -> Result<git2::Oid, CommitDiffPatchIdError> {
    let diff = commit_diff(repo, commit).map_err(CommitDiffPatchIdError::GetDiffFailed)?;
    diff.patchid(Option::None)
        .map_err(CommitDiffPatchIdError::CreatePatchHashFailed)
}

#[derive(Debug)]
pub enum CommonAncestorError {
    MergeBase(git2::Error),
    FindCommit(git2::Error),
    GetParentZero(git2::Error),
}

pub fn common_ancestor(
    repo: &git2::Repository,
    one: git2::Oid,
    two: git2::Oid,
) -> Result<git2::Oid, CommonAncestorError> {
    let merge_base_oid = repo
        .merge_base(one, two)
        .map_err(CommonAncestorError::MergeBase)?;
    Ok(merge_base_oid)
}

#[derive(Debug)]
pub enum UncommittedChangesError {
    StatusesFailed(git2::Error),
}

pub fn uncommitted_changes_exist(repo: &git2::Repository) -> Result<bool, UncommittedChangesError> {
    let mut status_options = git2::StatusOptions::default();
    status_options.show(git2::StatusShow::Workdir);
    status_options.include_untracked(true);
    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(UncommittedChangesError::StatusesFailed)?;
    Ok(!statuses.is_empty())
}

#[cfg(feature = "backup_cmd")]
#[derive(Debug)]
pub enum HashObjectWriteError {
    Failed(git2::Error),
}

#[cfg(feature = "backup_cmd")]
pub fn hash_object_write(
    repo: &git2::Repository,
    content: &str,
) -> Result<git2::Oid, HashObjectWriteError> {
    repo.blob(content.as_bytes())
        .map_err(HashObjectWriteError::Failed)
}

#[cfg(feature = "backup_cmd")]
#[derive(Debug)]
pub enum ReadHashedObjectError {
    NotValidUtf8(std::str::Utf8Error),
    Failed(git2::Error),
}

#[cfg(feature = "backup_cmd")]
pub fn read_hashed_object(
    repo: &git2::Repository,
    oid: git2::Oid,
) -> Result<String, ReadHashedObjectError> {
    let blob = repo.find_blob(oid).map_err(ReadHashedObjectError::Failed)?;
    let content = blob.content();
    let str_ref = std::str::from_utf8(content).map_err(ReadHashedObjectError::NotValidUtf8)?;
    Ok(str_ref.to_string())
}

#[cfg(test)]
mod tests {
    use git2::{Repository, RepositoryInitOptions};
    use tempfile::TempDir;

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

    pub fn create_commit(
        repo: &git2::Repository,
        path: &str,
        data: &[u8],
        message: &str,
    ) -> git2::Oid {
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
        repo.commit(
            Option::Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent_commit],
        )
        .unwrap()
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

        let start_oid_excluded =
            create_commit(&repo, "fileA.txt", &[0, 1, 2, 3], "starting numbers");
        create_commit(
            &repo,
            "fileB.txt",
            &[4, 5, 6, 7],
            "four, five, six, and seven",
        );
        let end_oid_included = create_commit(
            &repo,
            "fileC.txt",
            &[8, 9, 10, 11],
            "eight, nine, ten, and eleven",
        );
        create_commit(
            &repo,
            "fileD.txt",
            &[12, 13, 14, 15],
            "twelve, thirteen, forteen, fifteen",
        );

        let rev_walk = super::get_revs(
            &repo,
            start_oid_excluded,
            end_oid_included,
            git2::Sort::REVERSE,
        )
        .unwrap();
        let summaries: Vec<String> = rev_walk
            .map(|oid| {
                repo.find_commit(oid.unwrap())
                    .unwrap()
                    .summary()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(summaries.len(), 2);

        assert_eq!(summaries.first().unwrap(), "four, five, six, and seven");
        assert_eq!(summaries.last().unwrap(), "eight, nine, ten, and eleven");
    }

    #[cfg(feature = "backup_cmd")]
    #[test]
    fn test_hash_object_write() {
        let (_td, repo) = repo_init();
        let message = "Hello hash object write!";
        let oid = super::hash_object_write(&repo, message).unwrap();
        let blob = repo.find_blob(oid).unwrap();
        assert_eq!(blob.content(), message.as_bytes());
    }

    #[cfg(feature = "backup_cmd")]
    #[test]
    fn test_read_hashed_object() {
        let (_td, repo) = repo_init();
        let message = "Hello hash object write!";
        let oid = super::hash_object_write(&repo, message).unwrap();
        let retreived_message = super::read_hashed_object(&repo, oid).unwrap();
        assert_eq!(retreived_message, message);
    }
}
