use std::result::Result;
use super::super::private::utils;

#[derive(Debug)]
pub enum UnstageError {
  UnstageFailed(utils::ExecuteError)
}

pub fn unstage(files: Vec<std::string::String>) -> Result<(), UnstageError> {
  let args: Vec<&str> = ["reset"].to_vec();

  let files_strs: Vec<&str> = files.iter().map(|s| s as &str).collect();
  let final_args = [args, files_strs].concat();

  utils::execute("git", &final_args).map_err(UnstageError::UnstageFailed)
}
