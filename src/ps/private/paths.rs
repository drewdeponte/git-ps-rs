use git2;
use home_dir::{self, HomeDirExt};
use is_executable::IsExecutable;
use std::path::{Path, PathBuf};

const PATCH_STATES_RELATIVE_PATH: &str = "GIT-PATCH-STACK-PATCH-STATES-V3.json";
const ISOLATE_LAST_BRANCH_RELATIVE_PATH: &str = "GIT-PATCH-STACK-ISOLATE-LAST-BRANCH";

#[derive(Debug)]
pub enum PathsError {
    RepoWorkDirNotFound,
}

pub fn repo_root_path(repo: &git2::Repository) -> Result<&Path, PathsError> {
    repo.workdir().ok_or(PathsError::RepoWorkDirNotFound)
}

pub fn patch_states_path(repo: &git2::Repository) -> PathBuf {
    repo.path().join(PATCH_STATES_RELATIVE_PATH)
}

pub fn isolate_last_branch_path(repo: &git2::Repository) -> PathBuf {
    repo.path().join(ISOLATE_LAST_BRANCH_RELATIVE_PATH)
}

pub fn communal_repository_level_config_path(repo_root: &str) -> PathBuf {
    let path_string = format!("{}/.git-ps/config.toml", repo_root);
    Path::new(path_string.as_str()).to_path_buf()
}

pub fn personal_repository_level_config_path(repo_gitdir: &str) -> PathBuf {
    let path_string = format!("{}/git-ps/config.toml", repo_gitdir);
    Path::new(path_string.as_str()).to_path_buf()
}

#[derive(Debug)]
pub enum UserLevelConfigPathError {
    PathExpandHomeFailed(home_dir::Error),
}

pub fn user_level_config_path() -> Result<PathBuf, UserLevelConfigPathError> {
    let path_string = "~/.config/git-ps/config.toml".to_string();
    let path = Path::new(path_string.as_str())
        .expand_home()
        .map_err(UserLevelConfigPathError::PathExpandHomeFailed)?;
    Ok(path)
}

pub enum PathExistsAndIsExecutable {
    ExistsAndIsExecutable,
    DoesNotExist,
    ExistsButNotExecutable,
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
