use std::{path::{Path, PathBuf}, str::Utf8Error};
use super::{paths::{PathExistsAndIsExecutable, path_exists_and_is_executable, self}, utils};
use home_dir::{self, HomeDirExt};

#[derive(Debug)]
pub enum FindHookError {
  NotExecutable(PathBuf),
  PathExpandHomeFailed(home_dir::Error),
  NotFound
}

pub fn find_hook(repo_root: &str, filename: &str) -> Result<PathBuf, FindHookError> {
  let communal_repository_level_hook_path_string = format!("{}/.git-ps/hooks/{}", repo_root, filename);
  let communal_repository_level_hook_path = Path::new(communal_repository_level_hook_path_string.as_str());
  let repository_level_hook_path_string = format!("{}/.git/git-ps/hooks/{}", repo_root, filename);
  let repository_level_hook_path = Path::new(repository_level_hook_path_string.as_str());
  let user_level_hook_path_string = format!("~/.config/git-ps/hooks/{}", filename);
  let user_level_hook_path = Path::new(user_level_hook_path_string.as_str()).expand_home().map_err(FindHookError::PathExpandHomeFailed)?;
  match path_exists_and_is_executable(communal_repository_level_hook_path) {
    PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(communal_repository_level_hook_path.to_path_buf()),
    PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(communal_repository_level_hook_path.to_path_buf())),
    PathExistsAndIsExecutable::DoesNotExist => match path_exists_and_is_executable(repository_level_hook_path) {
      PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(repository_level_hook_path.to_path_buf()),
      PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(repository_level_hook_path.to_path_buf())),
      PathExistsAndIsExecutable::DoesNotExist => match path_exists_and_is_executable(&user_level_hook_path) {
        PathExistsAndIsExecutable::ExistsAndIsExecutable => Ok(user_level_hook_path.to_path_buf()),
        PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(user_level_hook_path.to_path_buf())),
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
  HookOutputInvalid(Utf8Error),
}

pub fn get_hook_output(repo: &git2::Repository, hook_name: &str, patch_args: &[&str]) -> Result<String, HookOutputError> {
  let repo_root_path = paths::repo_root_path(&repo).map_err(HookOutputError::GetRepoRootPathFailed)?;
  let repo_root_str = repo_root_path.to_str().ok_or(HookOutputError::PathNotUtf8)?;

  let hook_output = match find_hook(repo_root_str, hook_name) {
    Ok(hook_path) => utils::execute_with_output(hook_path.to_str().ok_or(HookOutputError::PathNotUtf8)?, patch_args).map_err(HookOutputError::HookExecutionFailed)?,
    Err(e) => return Err(HookOutputError::HookNotFound(e))
  };

  return String::from_utf8(hook_output.stdout).or_else(|e| Err(HookOutputError::HookOutputInvalid(e.utf8_error())));
}