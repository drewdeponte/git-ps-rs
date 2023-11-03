use super::super::super::ps;
use super::super::private::git;

#[derive(Debug)]
pub enum ShaError {
    PatchIndexNotFound,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ShaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatchIndexNotFound => write!(f, "patch with patch index not found"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ShaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PatchIndexNotFound => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn sha(patch_index: usize, exclude_newline: bool) -> Result<(), ShaError> {
    let repo = git::create_cwd_repo().map_err(|e| ShaError::Unhandled(e.into()))?;

    let patch_stack = ps::get_patch_stack(&repo).map_err(|e| ShaError::Unhandled(e.into()))?;
    let patches_vec =
        ps::get_patch_list(&repo, &patch_stack).map_err(|e| ShaError::Unhandled(e.into()))?;

    let patch_oid = patches_vec
        .get(patch_index)
        .ok_or(ShaError::PatchIndexNotFound)?
        .oid;

    if exclude_newline {
        print!("{}", patch_oid);
    } else {
        println!("{}", patch_oid);
    }

    Ok(())
}
