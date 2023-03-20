use super::super::private::utils;
use std::result::Result;

#[derive(Debug)]
pub enum StatusError {
    StatusFailed(utils::ExecuteError),
}

pub fn status() -> Result<(), StatusError> {
    utils::execute("git", &["status"]).map_err(StatusError::StatusFailed)?;
    Ok(())
}
