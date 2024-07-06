use super::super::private::git;
use std::result::Result;

#[derive(Debug)]
pub enum PushError {
    OpenRepositoryFailed(Box<dyn std::error::Error>),
    FindBranchFailed(Box<dyn std::error::Error>),
    GetUpstreamBranchFailed(Box<dyn std::error::Error>),
    GetUpstreamBranchNameFailed(Box<dyn std::error::Error>),
    RemoteBranchNameNotUtf8,
    ExtractRemoteFailed(String),
    ExtractRelativeBranchNameFailed(String),
    PushFailed(Box<dyn std::error::Error>),
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenRepositoryFailed(e) => write!(f, "failed to open repository {}", e),
            Self::FindBranchFailed(e) => {
                write!(f, "failed to find branch with the provided name {}", e)
            }
            Self::GetUpstreamBranchFailed(e) => write!(f, "failed to get upstream branch, {}", e),
            Self::GetUpstreamBranchNameFailed(e) => {
                write!(f, "failed to get upstream branch name, {}", e)
            }
            Self::RemoteBranchNameNotUtf8 => write!(f, "remote branch name isn't valid utf-8"),
            Self::ExtractRemoteFailed(from) => {
                write!(f, "failed to extract remote name from {}", from)
            }
            Self::ExtractRelativeBranchNameFailed(from) => {
                write!(f, "failed to extract relative branch name from {}", from)
            }
            Self::PushFailed(e) => write!(f, "failed when trying to push, {}", e),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for PushError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenRepositoryFailed(e) => Some(e.as_ref()),
            Self::FindBranchFailed(e) => Some(e.as_ref()),
            Self::GetUpstreamBranchFailed(e) => Some(e.as_ref()),
            Self::GetUpstreamBranchNameFailed(e) => Some(e.as_ref()),
            Self::RemoteBranchNameNotUtf8 => None,
            Self::ExtractRemoteFailed(_) => None,
            Self::ExtractRelativeBranchNameFailed(_) => None,
            Self::PushFailed(e) => Some(e.as_ref()),
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn push(branch_name: String) -> Result<(), PushError> {
    let repo = git::create_cwd_repo().map_err(|e| PushError::OpenRepositoryFailed(e.into()))?;

    let branch = repo
        .find_branch(&branch_name, git2::BranchType::Local)
        .map_err(|e| PushError::FindBranchFailed(e.into()))?;

    let remote_branch = branch
        .upstream()
        .map_err(|e| PushError::GetUpstreamBranchFailed(e.into()))?;

    let remote_branch_name_str = remote_branch
        .name()
        .map_err(|e| PushError::GetUpstreamBranchNameFailed(e.into()))?
        .ok_or(PushError::RemoteBranchNameNotUtf8)?;

    let mut remote_branch_name_parts = remote_branch_name_str.split('/');
    let remote_branch_remote =
        remote_branch_name_parts
            .next()
            .ok_or(PushError::ExtractRemoteFailed(
                remote_branch_name_str.to_owned(),
            ))?;
    let remote_branch_relative_name =
        remote_branch_name_parts
            .next()
            .ok_or(PushError::ExtractRelativeBranchNameFailed(
                remote_branch_name_str.to_owned(),
            ))?;

    // e.g. origin/the-branch so it is <remote>/<branch-name-on-remote>

    git::ext_push(
        false,
        remote_branch_remote,
        &branch_name,
        remote_branch_relative_name,
    )
    .map_err(|e| PushError::PushFailed(e.into()))?;

    Ok(())
}
