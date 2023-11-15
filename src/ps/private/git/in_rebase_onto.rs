use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum InRebaseOntoError {
    NotInRebase,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for InRebaseOntoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInRebase => write!(f, "not in the middle of a rebase"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for InRebaseOntoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotInRebase => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

/// Get the onto of the current rebase
///
/// Reads the onto from the .git/rebase-merge/onto data for the current rebase.
pub fn in_rebase_onto<P: AsRef<Path>>(repo_gitdir: P) -> Result<String, InRebaseOntoError> {
    let onto_path = repo_gitdir.as_ref().join("rebase-merge").join("onto");

    if onto_path.is_file() {
        fs::read_to_string(onto_path).map_err(|e| InRebaseOntoError::Unhandled(e.into()))
    } else {
        Err(InRebaseOntoError::NotInRebase)
    }
}
