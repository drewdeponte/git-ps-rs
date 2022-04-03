use super::plumbing::utils;
use std::io;

#[derive(Debug)]
pub enum PullError {
  ExitStatus(i32),
  ExitSignal(i32),
  IOError(io::Error),
  Unknown
}

impl From<utils::ExecuteError> for PullError {
  fn from(e: utils::ExecuteError) -> Self {
    match e {
      utils::ExecuteError::SpawnFailure(io_error) => PullError::IOError(io_error),
      utils::ExecuteError::Failure(io_error) => PullError::IOError(io_error),
      utils::ExecuteError::ExitStatus(code) => PullError::ExitStatus(code),
      utils::ExecuteError::ExitSignal(signal) => PullError::ExitSignal(signal),
      utils::ExecuteError::ExitMissingSignal => PullError::Unknown
    }
  }
}

pub fn pull() -> Result<(), PullError> {
  utils::execute("git", &["pull", "--rebase"]).map_err(PullError::from)
}
