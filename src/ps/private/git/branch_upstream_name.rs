use super::git_error::GitError;
use git2;
use std::result::Result;
use std::str;

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
