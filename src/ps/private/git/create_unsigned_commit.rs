use git2;
use std::result::Result;

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
