use super::super::utils;
use std::result::Result;

#[derive(Debug)]
pub enum ExtDeleteRemoteBranchError {
    ExecuteFailed(utils::ExecuteError),
}

impl std::fmt::Display for ExtDeleteRemoteBranchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecuteFailed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ExtDeleteRemoteBranchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ExecuteFailed(e) => Some(e),
        }
    }
}

pub fn ext_delete_remote_branch(
    remote_name: &str,
    branch_name: &str,
) -> Result<(), ExtDeleteRemoteBranchError> {
    let refspecs = format!(":{}", branch_name);
    utils::execute("git", &["push", remote_name, &refspecs])
        .map_err(ExtDeleteRemoteBranchError::ExecuteFailed)?;
    Ok(())
}
