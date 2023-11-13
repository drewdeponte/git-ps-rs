use super::git_error::GitError;
use git2;
use std::result::Result;

/// Get Commit Summary given a repository & oid
pub fn get_summary(repo: &git2::Repository, oid: &git2::Oid) -> Result<String, GitError> {
    Ok(String::from(
        repo.find_commit(*oid)?
            .summary()
            .ok_or(GitError::NotFound)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::repo_init;

    #[test]
    fn smoke_get_summary() {
        let (_td, repo) = repo_init();
        let head_id = repo.refname_to_id("HEAD").unwrap();

        let res = super::get_summary(&repo, &head_id);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "initial");
    }
}
