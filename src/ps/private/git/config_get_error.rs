#[derive(Debug)]
pub enum ConfigGetError {
    Failed(git2::Error),
}

impl std::fmt::Display for ConfigGetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ConfigGetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failed(e) => Some(e),
        }
    }
}
