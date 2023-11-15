use std::fs;
use std::path::Path;

use super::{str_to_rebase_todo, RebaseTodoCommand};

#[derive(Debug)]
pub enum InRebaseTodosError {
    NotInRebase,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for InRebaseTodosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInRebase => write!(f, "not in the middle of a rebase"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for InRebaseTodosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotInRebase => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

/// Get the todos of the current rebase
///
/// Reads the todos from the .git/rebase-merge/git-rebase-todo data for the current rebase.
pub fn in_rebase_todos<P: AsRef<Path>>(
    repo_gitdir: P,
) -> Result<Vec<RebaseTodoCommand>, InRebaseTodosError> {
    let todos_path = repo_gitdir
        .as_ref()
        .join("rebase-merge")
        .join("git-rebase-todo");

    if todos_path.is_file() {
        let todos_content =
            fs::read_to_string(todos_path).map_err(|e| InRebaseTodosError::Unhandled(e.into()))?;

        str_to_rebase_todo(&todos_content).map_err(|e| InRebaseTodosError::Unhandled(e.into()))
    } else {
        Err(InRebaseTodosError::NotInRebase)
    }
}
