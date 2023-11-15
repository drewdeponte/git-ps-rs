use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum InRebaseHeadNameError {
    NotInRebase,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for InRebaseHeadNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInRebase => write!(f, "not in the middle of a rebase"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for InRebaseHeadNameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotInRebase => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

/// Get the head name of the current rebase
///
/// Reads the head name from the .git/rebase-merge/head-name data for the current rebase.
pub fn in_rebase_head_name<P: AsRef<Path>>(
    repo_gitdir: P,
) -> Result<String, InRebaseHeadNameError> {
    let head_name_path = repo_gitdir.as_ref().join("rebase-merge").join("head-name");

    if head_name_path.is_file() {
        fs::read_to_string(head_name_path).map_err(|e| InRebaseHeadNameError::Unhandled(e.into()))
    } else {
        Err(InRebaseHeadNameError::NotInRebase)
    }
}
