use git2;
use std::result::Result;

#[derive(Debug)]
pub enum UncommittedChangesError {
    StatusesFailed(git2::Error),
}

impl std::fmt::Display for UncommittedChangesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StatusesFailed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for UncommittedChangesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StatusesFailed(e) => Some(e),
        }
    }
}

pub fn uncommitted_changes_exist(repo: &git2::Repository) -> Result<bool, UncommittedChangesError> {
    let mut status_options = git2::StatusOptions::default();
    status_options.show(git2::StatusShow::Workdir);
    status_options.include_untracked(true);
    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(UncommittedChangesError::StatusesFailed)?;
    Ok(!statuses.is_empty())
}
