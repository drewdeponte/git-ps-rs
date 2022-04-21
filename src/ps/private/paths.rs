use git2;
use std::path::{Path, PathBuf};
use is_executable::IsExecutable;

const PATCH_STATES_RELATIVE_PATH: &str = ".git/GIT-PATCH-STACK-PATCH-STATES-V1.json";
const ISOLATE_LAST_BRANCH_RELATIVE_PATH: &str = ".git/GIT-PATCH-STACK-ISOLATE-LAST-BRANCH";

#[derive(Debug)]
pub enum PathsError {
  RepoWorkDirNotFound
}

pub fn repo_root_path(repo: &git2::Repository) -> Result<&Path, PathsError> {
  Ok(repo.workdir().ok_or(PathsError::RepoWorkDirNotFound)?)
}

pub fn patch_states_path(repo: &git2::Repository) -> Result<PathBuf, PathsError> {
  repo_root_path(repo).map(|p| p.join(PATCH_STATES_RELATIVE_PATH))
}

pub fn isolate_last_branch_path(repo: &git2::Repository) -> Result<PathBuf, PathsError> {
  repo_root_path(repo).map(|p| p.join(ISOLATE_LAST_BRANCH_RELATIVE_PATH))
}

pub enum PathExistsAndIsExecutable {
  ExistsAndIsExecutable,
  DoesNotExist,
  ExistsButNotExecutable
}

pub fn path_exists_and_is_executable(path: &Path) -> PathExistsAndIsExecutable {
  if path.exists() {
    if path.is_executable() {
      PathExistsAndIsExecutable::ExistsAndIsExecutable
    } else {
      PathExistsAndIsExecutable::ExistsButNotExecutable
   }
  } else {
    PathExistsAndIsExecutable::DoesNotExist
  }
}
