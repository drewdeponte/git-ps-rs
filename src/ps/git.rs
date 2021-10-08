#[derive(Debug)]
pub enum GetSummaryError {
    GitError(git2::Error),
    NotFound
}

impl From<git2::Error> for GetSummaryError {
    fn from(e: git2::Error) -> Self {
        Self::GitError(e)
    }
}

/// Get Commit Summary given a repository & oid
pub fn get_summary(repo: &git2::Repository, oid: &git2::Oid) -> Result<String, GetSummaryError>{
    Ok(String::from(repo.find_commit(*oid)?
                        .summary().ok_or(GetSummaryError::NotFound)?))
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_summary() {
        let (_td, repo) = crate::ps::test::repo_init();
        let head_id = repo.refname_to_id("HEAD").unwrap();

        let res = super::get_summary(&repo, &head_id);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "initial");
    }
}
