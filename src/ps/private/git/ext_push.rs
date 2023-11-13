use super::super::utils;
use std::result::Result;
use std::str;

#[derive(Debug)]
pub enum ExtForcePushError {
    ExecuteFailed(utils::ExecuteError),
}

impl std::fmt::Display for ExtForcePushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecuteFailed(e) => write!(f, "external force push failed, {}", e),
        }
    }
}

impl std::error::Error for ExtForcePushError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ExecuteFailed(e) => Some(e),
        }
    }
}

pub fn ext_push(
    force: bool,
    remote_name: &str,
    src_ref_spec: &str,
    dest_ref_spec: &str,
) -> Result<(), ExtForcePushError> {
    let refspecs = format!("{}:{}", src_ref_spec, dest_ref_spec);
    if force {
        utils::execute("git", &["push", "-f", remote_name, &refspecs])
            .map_err(ExtForcePushError::ExecuteFailed)
    } else {
        utils::execute("git", &["push", remote_name, &refspecs])
            .map_err(ExtForcePushError::ExecuteFailed)
    }
}
