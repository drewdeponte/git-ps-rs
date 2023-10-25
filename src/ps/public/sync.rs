use super::super::super::ps;
use super::super::private::git;

#[derive(Debug)]
pub enum SyncError {
    RepositoryNotFound,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetPatchStackBranchRemoteNameFailed(Box<dyn std::error::Error>),
    MergeCommitDetected(String),
    ConflictsExist(String, String),
    PatchBranchNameMissing,
    PatchUpstreamBranchNameMissing,
    BranchRemoteNameNotUtf8,
    SetPatchBranchUpstreamFailed(Box<dyn std::error::Error>),
    ForcePushFailed(Box<dyn std::error::Error>),
    GetBranchUpstreamRemoteName(Box<dyn std::error::Error>),
    PatchBranchRefMissing,
    Unhandled(Box<dyn std::error::Error>),
}

impl From<ps::private::branch::BranchError> for SyncError {
    fn from(value: ps::private::branch::BranchError) -> Self {
        match value {
            ps::private::branch::BranchError::MergeCommitDetected(oid) => {
                Self::MergeCommitDetected(oid)
            }
            ps::private::branch::BranchError::ConflictsExist(src_oid, dst_oid) => {
                Self::ConflictsExist(src_oid, dst_oid)
            }
            _ => Self::Unhandled(value.into()),
        }
    }
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepositoryNotFound => write!(f, "repository not found"),
            Self::CurrentBranchNameMissing => write!(f, "current branch name missing"),
            Self::GetUpstreamBranchNameFailed => write!(f, "failed to get upstream branch name"),
            Self::GetPatchStackBranchRemoteNameFailed(e) => {
                write!(f, "failed to get patch stack branch remote name, {}", e)
            }
            Self::MergeCommitDetected(oid) => write!(f, "merge commit detected with sha {}", oid),
            Self::ConflictsExist(src_oid, dst_oid) => write!(
                f,
                "conflict found when playing {} on top of {}",
                src_oid, dst_oid
            ),
            Self::PatchBranchNameMissing => write!(f, "patch branch name missing"),
            Self::PatchUpstreamBranchNameMissing => write!(f, "patch upstream branch name missing"),
            Self::BranchRemoteNameNotUtf8 => write!(f, "branch remote name is not utf-8"),
            Self::SetPatchBranchUpstreamFailed(e) => {
                write!(f, "failed to set patch branch upstream, {}", e)
            }
            Self::ForcePushFailed(e) => write!(f, "failed to force push patch up to remote, {}", e),
            Self::GetBranchUpstreamRemoteName(e) => {
                write!(f, "failed to get branch upstream remote name, {}", e)
            }
            Self::PatchBranchRefMissing => write!(f, "patch branch ref missing"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for SyncError {}

pub fn sync(
    start_patch_index: usize,
    end_patch_index: Option<usize>,
    given_branch_name: Option<String>,
) -> Result<(String, String), SyncError> {
    let repo = git::create_cwd_repo().map_err(|_| SyncError::RepositoryNotFound)?;

    // get remote name of current branch
    let cur_patch_stack_branch_name =
        git::get_current_branch(&repo).ok_or(SyncError::CurrentBranchNameMissing)?;
    let cur_patch_stack_branch_upstream_name =
        git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
            .map_err(|_| SyncError::GetUpstreamBranchNameFailed)?;
    let cur_patch_stack_remote_name = repo
        .branch_remote_name(&cur_patch_stack_branch_upstream_name)
        .map_err(|e| SyncError::GetPatchStackBranchRemoteNameFailed(e.into()))?;
    let cur_patch_stack_remote_name_str: &str = cur_patch_stack_remote_name
        .as_str()
        .ok_or(SyncError::BranchRemoteNameNotUtf8)?;

    // create request review branch for patch
    let (mut patch_branch, _new_commit_oid) =
        ps::private::branch::branch(&repo, start_patch_index, end_patch_index, given_branch_name)?;

    // get upstream branch name & remote of patch branch or fallback to using patch branch name &
    // cur patch stack remote.
    let patch_branch_name: String = patch_branch
        .get()
        .shorthand()
        .ok_or(SyncError::PatchBranchNameMissing)?
        .to_owned();

    let (upstream_patch_branch_name, upstream_patch_remote_name) = match patch_branch.upstream() {
        Ok(upstream_branch) => {
            let upstream_branch_name = upstream_branch
                .get()
                .shorthand()
                .map(|n| n.to_string())
                .ok_or(SyncError::PatchUpstreamBranchNameMissing)?;

            let patch_branch_ref: String = patch_branch
                .get()
                .name()
                .ok_or(SyncError::PatchBranchRefMissing)?
                .to_owned();

            // get the configured remote of the patch branch
            let remote_name: String = repo
                .branch_upstream_remote(&patch_branch_ref)
                .map_err(|e| SyncError::GetBranchUpstreamRemoteName(e.into()))?
                .as_str()
                .ok_or(SyncError::BranchRemoteNameNotUtf8)?
                .to_string();

            // strip the origin/ off the front of the name (origin/foo) to get the remote relative name
            let pattern = format!("{}/", &remote_name);
            let upstream_branch_name_relative_to_remote =
                str::replace(&upstream_branch_name, pattern.as_str(), "");

            git::ext_push(
                true,
                &remote_name,
                &patch_branch_name,
                &upstream_branch_name_relative_to_remote,
            )
            .map_err(|e| SyncError::ForcePushFailed(e.into()))?;

            (upstream_branch_name_relative_to_remote, remote_name)
        }
        Err(_e) => {
            git::ext_push(
                true,
                cur_patch_stack_remote_name_str,
                &patch_branch_name,
                &patch_branch_name,
            )
            .map_err(|e| SyncError::ForcePushFailed(e.into()))?;

            patch_branch
                .set_upstream(Some(
                    format!("{}/{}", cur_patch_stack_remote_name_str, &patch_branch_name).as_str(),
                ))
                .map_err(|e| SyncError::SetPatchBranchUpstreamFailed(e.into()))?;

            (
                patch_branch_name.to_string(),
                cur_patch_stack_remote_name_str.to_string(),
            )
        }
    };

    Ok((
        upstream_patch_branch_name.to_string(),
        upstream_patch_remote_name,
    ))
}
