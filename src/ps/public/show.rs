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
