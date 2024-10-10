use git2;
use std::result::Result;

#[derive(Debug)]
pub enum CommonAncestorError {
    MergeBase { one: git2::Oid, two: git2::Oid, reason: git2::Error }
}

impl std::fmt::Display for CommonAncestorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MergeBase { one, two, reason } => write!(f, "failed to get merge base for {} and {}, {}", one, two, reason),
        }
    }
}

impl std::error::Error for CommonAncestorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MergeBase { one: _, two: _, reason } => Some(reason),
        }
    }
}

pub fn common_ancestor(
    repo: &git2::Repository,
    one: git2::Oid,
    two: git2::Oid,
) -> Result<git2::Oid, CommonAncestorError> {
    let merge_base_oid = repo
        .merge_base(one, two)
        .map_err(|e| CommonAncestorError::MergeBase { one: one, two: two, reason: e })?;
    Ok(merge_base_oid)
}
