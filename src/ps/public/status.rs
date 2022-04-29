use std::result::Result;
use super::super::private::utils;

#[derive(Debug)]
pub enum StatusError {
  StatusFailed(utils::ExecuteError)
}

pub fn status() -> Result<(), StatusError>  {
  utils::execute("git", &["status"]).map_err(StatusError::StatusFailed)?;
  Ok(())
}
