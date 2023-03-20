use super::super::private::utils;
use std::result::Result;
use std::string::String;

#[derive(Debug)]
pub enum AddChangesToStageError {
    AddFailed(utils::ExecuteError),
}

pub fn add_changes_to_stage(
    interactive: bool,
    patch: bool,
    edit: bool,
    all: bool,
    files: Vec<String>,
) -> Result<(), AddChangesToStageError> {
    let mut args: Vec<&str> = ["add"].to_vec();

    if interactive {
        args.push("--interactive");
    } else if patch {
        args.push("--patch");
    } else if edit {
        args.push("--edit");
    }

    if all {
        args.push("--all");
    }

    let files_strs: Vec<&str> = files.iter().map(|s| s as &str).collect();
    let final_args = [args, files_strs].concat();

    utils::execute("git", &final_args).map_err(AddChangesToStageError::AddFailed)?;

    Ok(())
}
