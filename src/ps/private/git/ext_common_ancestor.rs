use super::super::utils;
use git2;
use std::result::Result;

#[derive(Debug)]
pub enum ExtCommonAncestorError {
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ExtCommonAncestorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ExtCommonAncestorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

impl From<utils::ExecuteWithOutputError> for ExtCommonAncestorError {
    fn from(value: utils::ExecuteWithOutputError) -> Self {
        Self::Unhandled(value.into())
    }
}

impl From<std::string::FromUtf8Error> for ExtCommonAncestorError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Unhandled(value.into())
    }
}

impl From<git2::Error> for ExtCommonAncestorError {
    fn from(value: git2::Error) -> Self {
        Self::Unhandled(value.into())
    }
}

pub fn ext_common_ancestor(
    one: git2::Oid,
    two: git2::Oid,
) -> Result<git2::Oid, ExtCommonAncestorError> {
    let output =
        utils::execute_with_output("git", &["merge-base", &one.to_string(), &two.to_string()])?;
    let sha = String::from_utf8(output.stdout)?;
    let common_ancestor_oid = git2::Oid::from_str(sha.trim())?;
    Ok(common_ancestor_oid)
}
