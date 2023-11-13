use super::signers;
use git2;
use std::result::Result;
use std::str;

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
