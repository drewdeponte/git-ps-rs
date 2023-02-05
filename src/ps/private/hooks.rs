use std::{path::{Path, PathBuf}, process::Output};
use super::{paths::{PathExistsAndIsExecutable, path_exists_and_is_executable, self}, utils};
use home_dir::{self, HomeDirExt};

#[derive(Debug)]
pub enum FindHookError {
  NotExecutable(PathBuf),
  PathExpandHomeFailed(home_dir::Error),
  NotFound
}

pub fn find_hook(repo_root: &str, filename: &str) -> Result<PathBuf, FindHookError> {
  let communal_repository_level_hook_pathbuf: PathBuf = [repo_root, ".git-ps", "hooks", filename].iter().collect();
  let repository_level_hook_pathbuf: PathBuf = [repo_root, ".git", "git-ps", "hooks", filename].iter().collect();
  let mut user_level_hook_pathbuf = "~".expand_home().map_err(FindHookError::PathExpandHomeFailed)?;
  user_level_hook_pathbuf.push(".config");
  user_level_hook_pathbuf.push("git-ps");
  user_level_hook_pathbuf.push("hooks");
  user_level_hook_pathbuf.push(filename);
  match path_exists_and_is_executable(communal_repository_level_hook_pathbuf.as_path()) {
    PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(communal_repository_level_hook_pathbuf),
    PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(communal_repository_level_hook_pathbuf)),
    PathExistsAndIsExecutable::DoesNotExist => match path_exists_and_is_executable(repository_level_hook_pathbuf.as_path()) {
      PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(repository_level_hook_pathbuf),
      PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(repository_level_hook_pathbuf)),
      PathExistsAndIsExecutable::DoesNotExist => match path_exists_and_is_executable(&user_level_hook_pathbuf.as_path()) {
        PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(user_level_hook_pathbuf),
        PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(user_level_hook_pathbuf)),
        PathExistsAndIsExecutable::DoesNotExist => Err(FindHookError::NotFound)
      }
    }
  }
}

#[derive(Debug)]
pub enum HookOutputError {
  GetRepoRootPathFailed(paths::PathsError),
  PathNotUtf8,
  HookExecutionFailed(utils::ExecuteWithOutputError),
  HookNotFound(FindHookError),
}

pub fn find_and_execute_hook_with_output(repo_root_str: &str, hook_name: &str, hook_args: &[&str]) -> Result<Output, HookOutputError> {
  let hook_output = match find_hook(repo_root_str, hook_name) {
    Ok(hook_path) => utils::execute_with_output(hook_path.to_str().ok_or(HookOutputError::PathNotUtf8)?, hook_args).map_err(HookOutputError::HookExecutionFailed)?,
    Err(e) => return Err(HookOutputError::HookNotFound(e))
  };

  return Ok(hook_output);
}