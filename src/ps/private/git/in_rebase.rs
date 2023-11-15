use std::path::Path;

/// Check if we are in the middle of a rebase
///
/// Checks to see if we are in the middle of a rebase given a path to the .git directory within the
/// repository and return `true` if we are or `false` if we aren't.
pub fn in_rebase<P: AsRef<Path>>(repo_gitdir: P) -> bool {
    repo_gitdir.as_ref().join("rebase-merge").is_dir()
}
