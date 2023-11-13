#[derive(Debug)]
pub enum GitError {
    Git(git2::Error),
    NotFound,
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        Self::Git(e)
    }
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git(e) => write!(f, "{}", e),
            Self::NotFound => write!(f, "not found"),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Git(e) => Some(e),
            Self::NotFound => None,
        }
    }
}
