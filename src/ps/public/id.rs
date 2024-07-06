use super::super::super::ps;
use super::super::private::git;

#[derive(Debug)]
pub enum IdError {
    OpenGitConfigFailed(Box<dyn std::error::Error>),
    AddPatchIdsFailed(Box<dyn std::error::Error>),
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for IdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenGitConfigFailed(e) => {
                write!(f, "Failed to open git config, {}", e)
            }
            Self::AddPatchIdsFailed(e) => write!(f, "add patch ids failed, {}", e),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for IdError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OpenGitConfigFailed(e) => Some(e.as_ref()),
            Self::AddPatchIdsFailed(e) => Some(e.as_ref()),
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn id() -> Result<(), IdError> {
    let repo = git::create_cwd_repo().map_err(|e| IdError::Unhandled(e.into()))?;

    let config =
        git2::Config::open_default().map_err(|e| IdError::OpenGitConfigFailed(e.into()))?;

    ps::add_patch_ids(&repo, &config).map_err(|e| IdError::AddPatchIdsFailed(e.into()))
}
