use super::super::private::utils;
use super::super::private::git;
use std::io;

#[derive(Debug)]
pub enum PullError {
  ExitStatus(i32),
  ExitSignal(i32),
  IOError(io::Error),
  RepositoryMissing,
  GetHeadBranchNameFailed,
  GetUpstreamBranchNameFailed,
  RebaseFailed(utils::ExecuteError),
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
  let repo = git::create_cwd_repo().map_err(|_| PullError::RepositoryMissing)?;

  let head_ref = repo.head().map_err(|_| PullError::GetHeadBranchNameFailed)?;
  let head_branch_shorthand = head_ref.shorthand().ok_or(PullError::GetHeadBranchNameFailed)?;
  let head_branch_name = head_ref.name().ok_or(PullError::GetHeadBranchNameFailed)?;

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).map_err(|_| PullError::GetUpstreamBranchNameFailed)?;

  utils::execute("git", &["fetch"]).map_err(PullError::from)?;

  utils::execute("git", &["rebase", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]).map_err(PullError::RebaseFailed)
}
