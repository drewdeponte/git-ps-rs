use super::super::super::ps;
use super::super::private::git;
use uuid::Uuid;

#[derive(Debug)]
pub enum SyncError {
    RepositoryNotFound,
    CurrentBranchNameMissing,
    GetUpstreamBranchNameFailed,
    GetPatchStackBranchRemoteNameFailed(git2::Error),
    CreateRrBranchFailed(ps::private::request_review_branch::RequestReviewBranchError),
    PatchBranchNameMissing,
    PatchUpstreamBranchNameMissing,
    BranchRemoteNameNotUtf8,
    SetPatchBranchUpstreamFailed(git2::Error),
    ForcePushFailed(git::ExtForcePushError),
    GetBranchUpstreamRemoteName(git2::Error),
    PatchBranchRefMissing,
}

pub fn sync(
    patch_index: usize,
    given_branch_name: Option<String>,
) -> Result<(String, String, Uuid), SyncError> {
    let repo = git::create_cwd_repo().map_err(|_| SyncError::RepositoryNotFound)?;

    // get remote name of current branch
    let cur_patch_stack_branch_name =
        git::get_current_branch(&repo).ok_or(SyncError::CurrentBranchNameMissing)?;
    let cur_patch_stack_branch_upstream_name =
        git::branch_upstream_name(&repo, cur_patch_stack_branch_name.as_str())
            .map_err(|_| SyncError::GetUpstreamBranchNameFailed)?;
    let cur_patch_stack_remote_name = repo
        .branch_remote_name(&cur_patch_stack_branch_upstream_name)
        .map_err(SyncError::GetPatchStackBranchRemoteNameFailed)?;
    let cur_patch_stack_remote_name_str: &str = cur_patch_stack_remote_name
        .as_str()
        .ok_or(SyncError::BranchRemoteNameNotUtf8)?;

    // create request review branch for patch
    let (mut patch_branch, ps_id, _new_commit_oid) =
        ps::private::request_review_branch::request_review_branch(
            &repo,
            patch_index,
            given_branch_name,
        )
        .map_err(SyncError::CreateRrBranchFailed)?;

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
                .map_err(SyncError::GetBranchUpstreamRemoteName)?
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
            .map_err(SyncError::ForcePushFailed)?;

            (upstream_branch_name_relative_to_remote, remote_name)
        }
        Err(_e) => {
            git::ext_push(
                true,
                cur_patch_stack_remote_name_str,
                &patch_branch_name,
                &patch_branch_name,
            )
            .map_err(SyncError::ForcePushFailed)?;

            patch_branch
                .set_upstream(Some(
                    format!("{}/{}", cur_patch_stack_remote_name_str, &patch_branch_name).as_str(),
                ))
                .map_err(SyncError::SetPatchBranchUpstreamFailed)?;

            (
                patch_branch_name.to_string(),
                cur_patch_stack_remote_name_str.to_string(),
            )
        }
    };

    Ok((
        upstream_patch_branch_name.to_string(),
        upstream_patch_remote_name,
        ps_id,
    ))
}
