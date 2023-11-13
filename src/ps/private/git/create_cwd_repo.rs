use git2;
use std::result::Result;

#[derive(Debug)]
pub enum CreateCwdRepositoryError {
    Failed(git2::Error),
}

impl From<git2::Error> for CreateCwdRepositoryError {
    fn from(e: git2::Error) -> Self {
        Self::Failed(e)
    }
}

impl std::fmt::Display for CreateCwdRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CreateCwdRepositoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(e) => Some(e),
        }
    }
}

/// Attempt to open an already-existing repository at or above current working
/// directory
pub fn create_cwd_repo() -> Result<git2::Repository, CreateCwdRepositoryError> {
    let repo = git2::Repository::discover("./")?;
    Ok(repo)
}
