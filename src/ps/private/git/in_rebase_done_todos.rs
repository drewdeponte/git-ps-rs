use std::fs;
use std::path::Path;

use super::{str_to_rebase_todo, RebaseTodoCommand};

#[derive(Debug)]
pub enum InRebaseDoneTodosError {
    NotInRebase,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for InRebaseDoneTodosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInRebase => write!(f, "not in the middle of a rebase"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for InRebaseDoneTodosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotInRebase => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

/// Get the done todos of the current rebase
///
/// Reads the done todos from the .git/rebase-merge/done data for the current rebase.
pub fn in_rebase_done_todos<P: AsRef<Path>>(
    repo_gitdir: P,
) -> Result<Vec<RebaseTodoCommand>, InRebaseDoneTodosError> {
    let done_path = repo_gitdir.as_ref().join("rebase-merge").join("done");

    if done_path.is_file() {
        let done_content = fs::read_to_string(done_path)
            .map_err(|e| InRebaseDoneTodosError::Unhandled(e.into()))?;

        str_to_rebase_todo(&done_content).map_err(|e| InRebaseDoneTodosError::Unhandled(e.into()))
    } else {
        Err(InRebaseDoneTodosError::NotInRebase)
    }
}
