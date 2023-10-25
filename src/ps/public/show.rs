use super::super::super::ps;
use super::super::private::git;
use super::super::private::utils;
use std::io;

#[derive(Debug)]
pub enum ShowError {
    ExitStatus(i32),
    ExitSignal(i32),
    IOError(io::Error),
    Unknown,
    RepositoryMissing,
    GetPatchStackFailed(ps::PatchStackError),
    GetPatchListFailed(ps::GetPatchListError),
    PatchIndexNotFound,
}

impl std::fmt::Display for ShowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExitStatus(status) => write!(f, "show exited with status {}", status),
            Self::ExitSignal(signal) => write!(f, "show exited with signal {}", signal),
            Self::IOError(e) => write!(f, "{}", e),
            Self::Unknown => write!(f, "Unknown failure"),
            Self::RepositoryMissing => write!(f, "repository missing"),
            Self::GetPatchStackFailed(e) => write!(f, "get patch stack failed, {}", e),
            Self::GetPatchListFailed(e) => {
                write!(f, "get patch stack list of patches failed, {}", e)
            }
            Self::PatchIndexNotFound => write!(f, "patch index not found"),
        }
    }
}

impl std::error::Error for ShowError {}

impl From<utils::ExecuteError> for ShowError {
    fn from(e: utils::ExecuteError) -> Self {
        match e {
            utils::ExecuteError::SpawnFailure(io_error) => Self::IOError(io_error),
            utils::ExecuteError::Failure(io_error) => Self::IOError(io_error),
            utils::ExecuteError::ExitStatus(code) => Self::ExitStatus(code),
            utils::ExecuteError::ExitSignal(signal) => Self::ExitSignal(signal),
            utils::ExecuteError::ExitMissingSignal => Self::Unknown,
        }
    }
}

pub fn show(start_patch_index: usize, end_patch_index: Option<usize>) -> Result<(), ShowError> {
    let repo = git::create_cwd_repo().map_err(|_| ShowError::RepositoryMissing)?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(ShowError::GetPatchStackFailed)?;
    let patches_vec =
        ps::get_patch_list(&repo, &patch_stack).map_err(ShowError::GetPatchListFailed)?;

    let start_patch_oid = patches_vec
        .get(start_patch_index)
        .ok_or(ShowError::PatchIndexNotFound)?
        .oid;

    if let Some(end_index) = end_patch_index {
        let end_patch_oid = patches_vec
            .get(end_index)
            .ok_or(ShowError::PatchIndexNotFound)?
            .oid;

        utils::execute(
            "git",
            &[
                "show",
                "--pretty=raw",
                format!("{}^...{}", start_patch_oid, end_patch_oid).as_str(),
            ],
        )
        .map_err(ShowError::from)
    } else {
        utils::execute(
            "git",
            &[
                "show",
                "--pretty=raw",
                format!("{}", start_patch_oid).as_str(),
            ],
        )
        .map_err(ShowError::from)
    }
}
