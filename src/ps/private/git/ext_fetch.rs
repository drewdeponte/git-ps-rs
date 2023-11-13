use super::super::utils;
use std::result::Result;

#[derive(Debug)]
pub enum ExtFetchError {
    ExecuteFailed(utils::ExecuteError),
}

impl std::fmt::Display for ExtFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecuteFailed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ExtFetchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ExecuteFailed(e) => Some(e),
        }
    }
}

pub fn ext_fetch() -> Result<(), ExtFetchError> {
    utils::execute("git", &["fetch"]).map_err(ExtFetchError::ExecuteFailed)?;
    Ok(())
}
