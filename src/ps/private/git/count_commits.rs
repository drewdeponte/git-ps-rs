use git2;
use std::result::Result;

#[derive(Debug)]
pub enum CountCommitsError {
    UnhandledError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for CountCommitsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnhandledError(boxed_e) => write!(f, "{}", boxed_e),
        }
    }
}

impl From<git2::Error> for CountCommitsError {
    fn from(e: git2::Error) -> Self {
        Self::UnhandledError(e.into())
    }
}

impl std::error::Error for CountCommitsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnhandledError(boxed_e) => Some(boxed_e.as_ref()), // Self::MergeBase(e) => Some(e),
        }
    }
}

/// Returns count of commits between `from_oid` (inclusive) and `to_oid` (exclusive).
///
/// `to_oid` should be an ancestor of `from_oid`
pub fn count_commits(
    repo: &git2::Repository,
    from_oid: git2::Oid,
    to_oid: git2::Oid,
) -> Result<usize, CountCommitsError> {
    let mut rev_walk = repo.revwalk()?;
    rev_walk.push(from_oid)?;
    rev_walk.hide(to_oid)?;
    rev_walk.set_sorting(git2::Sort::REVERSE)?;

    Ok(rev_walk.count())
}
