use super::{
    paths::{path_exists_and_is_executable, PathExistsAndIsExecutable},
    utils,
};
use std::{path::PathBuf, process::Output};

#[derive(Debug)]
pub enum FindHookError {
    NotExecutable(PathBuf),
    PathExpandHomeFailed(homedir::GetHomeError),
    HomeDirNotFound,
    NotFound,
}

impl std::fmt::Display for FindHookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotExecutable(path_buf) => {
                write!(
                    f,
                    "Hook at {} is not executable",
                    path_buf.to_str().unwrap_or("some non utf-8 path")
                )
            }
            Self::PathExpandHomeFailed(e) => {
                write!(f, "Failed to get home directory {}", e)
            }
            Self::HomeDirNotFound => write!(f, "Home directory not found"),
            Self::NotFound => write!(f, "Not found"),
        }
    }
}

impl std::error::Error for FindHookError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotExecutable(_) => None,
            Self::PathExpandHomeFailed(e) => Some(e),
            Self::HomeDirNotFound => None,
            Self::NotFound => None,
        }
    }
}

pub fn find_hook(
    repo_root: &str,
    repo_gitdir: &str,
    filename: &str,
) -> Result<PathBuf, FindHookError> {
    let communal_repository_level_hook_pathbuf: PathBuf =
        [repo_root, ".git-ps", "hooks", filename].iter().collect();
    let repository_level_hook_pathbuf: PathBuf =
        [repo_gitdir, "git-ps", "hooks", filename].iter().collect();
    let mut user_level_hook_pathbuf: PathBuf = homedir::my_home()
        .map_err(FindHookError::PathExpandHomeFailed)?
        .ok_or(FindHookError::HomeDirNotFound)?;
    user_level_hook_pathbuf.push(".config");
    user_level_hook_pathbuf.push("git-ps");
    user_level_hook_pathbuf.push("hooks");
    user_level_hook_pathbuf.push(filename);
    match path_exists_and_is_executable(communal_repository_level_hook_pathbuf.as_path()) {
        PathExistsAndIsExecutable::ExistsAndIsExecutable => {
            Ok(communal_repository_level_hook_pathbuf)
        }
        PathExistsAndIsExecutable::ExistsButNotExecutable => Err(FindHookError::NotExecutable(
            communal_repository_level_hook_pathbuf,
        )),
        PathExistsAndIsExecutable::DoesNotExist => {
            match path_exists_and_is_executable(repository_level_hook_pathbuf.as_path()) {
                PathExistsAndIsExecutable::ExistsAndIsExecutable => {
                    Ok(repository_level_hook_pathbuf)
                }
                PathExistsAndIsExecutable::ExistsButNotExecutable => {
                    Err(FindHookError::NotExecutable(repository_level_hook_pathbuf))
                }
                PathExistsAndIsExecutable::DoesNotExist => {
                    match path_exists_and_is_executable(user_level_hook_pathbuf.as_path()) {
                        PathExistsAndIsExecutable::ExistsAndIsExecutable => {
                            Ok(user_level_hook_pathbuf)
                        }
                        PathExistsAndIsExecutable::ExistsButNotExecutable => {
                            Err(FindHookError::NotExecutable(user_level_hook_pathbuf))
                        }
                        PathExistsAndIsExecutable::DoesNotExist => Err(FindHookError::NotFound),
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum HookOutputError {
    PathNotUtf8,
    HookExecutionFailed(utils::ExecuteWithOutputError),
    HookNotFound(FindHookError),
}

impl std::fmt::Display for HookOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathNotUtf8 => write!(f, "path not utf-8"),
            Self::HookExecutionFailed(e) => write!(f, "hook execution failed, {}", e),
            Self::HookNotFound(e) => write!(f, "hook not found, {}", e),
        }
    }
}

impl std::error::Error for HookOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PathNotUtf8 => None,
            Self::HookExecutionFailed(e) => Some(e),
            Self::HookNotFound(e) => Some(e),
        }
    }
}

pub fn find_and_execute_hook_with_output(
    repo_root_str: &str,
    repo_gitdir_str: &str,
    hook_name: &str,
    hook_args: &[&str],
) -> Result<Output, HookOutputError> {
    let hook_output = match find_hook(repo_root_str, repo_gitdir_str, hook_name) {
        Ok(hook_path) => utils::execute_with_output(
            hook_path.to_str().ok_or(HookOutputError::PathNotUtf8)?,
            hook_args,
        )
        .map_err(HookOutputError::HookExecutionFailed)?,
        Err(e) => return Err(HookOutputError::HookNotFound(e)),
    };

    Ok(hook_output)
}
