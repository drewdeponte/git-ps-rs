use super::super::private::git;
use super::super::super::ps;
use super::super::private::state_management;

#[derive(Debug)]
pub enum NewIntegrateError {
  RepositoryNotFound,
  FindPatchCommitFailed(ps::FindPatchCommitError),
  CommitPsIdMissing,
  GetPatchMetaDataFailed(ps::GetPatchMetaDataError),
  PatchMetaDataMissing,
  PatchHasNotBeenPushed,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteNameFailed,
  GetHeadFailed(git2::Error),
  HeadMissingTarget,
  ConvertStringToStrFailed,
  FindRemoteRrBranchFailed(git2::Error),
  RemoteRrBranchMissingTarget,
  CommonAncestorFailed(git::CommonAncestorError),
  GetRevWalkerFailed(git::GitError),
  PatchBranchDoesntHaveExactlyOneCommit(String, usize), // (branch_name, num_of_commits)
  FindRrBranchCommitFailed(git2::Error),
  RrBranchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchCommitDiffPatchIdFailed(git::CommitDiffPatchIdError),
  PatchesDiffer,
  PushFailed(ps::private::git::ExtForcePushError),
  UpdatePatchMetaDataFailed(state_management::StorePatchStateError),
  DeleteLocalBranchFailed(git2::Error),
  DeleteRemoteBranchFailed(git::ExtDeleteRemoteBranchError)
}

pub fn new_integrate(patch_index: usize, keep_branch: bool, given_branch_name: Option<String>) -> Result<(), NewIntegrateError> {
  let repo = git::create_cwd_repo().map_err(|_| NewIntegrateError::RepositoryNotFound)?;

  // verify that the patch-index has a corresponding commit
  let patch_commit = ps::find_patch_commit(&repo, patch_index).map_err(NewIntegrateError::FindPatchCommitFailed)?;

  // verify that the commit has a patch stack id
  let ps_id = ps::commit_ps_id(&patch_commit).ok_or(NewIntegrateError::CommitPsIdMissing)?;

  // verify that the patch has an associated branch and has been synced
  let patch_meta_data = ps::get_patch_meta_data(&repo, ps_id).map_err(NewIntegrateError::GetPatchMetaDataFailed)?.ok_or(NewIntegrateError::PatchMetaDataMissing)?;
  if !patch_meta_data.state.has_been_pushed_to_remote() {
    return Err(NewIntegrateError::PatchHasNotBeenPushed)
  }

  // TODO: verify that the patch has been requested-review

  // verify remote request-review branch has exactly one commit
  let rr_branch_name = patch_meta_data.state.branch_name();

  let cur_branch_name = git::get_current_branch(&repo).ok_or(NewIntegrateError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| NewIntegrateError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| NewIntegrateError::GetRemoteNameFailed)?;

  let remote_name_str = remote_name.as_str().ok_or(NewIntegrateError::ConvertStringToStrFailed)?;
  let mainline_head_oid = repo.head().map_err(NewIntegrateError::GetHeadFailed)?.target().ok_or(NewIntegrateError::HeadMissingTarget)?;
  let remote_rr_branch_refspec = format!("{}/{}", remote_name_str, rr_branch_name.as_str());
  let rr_branch_oid = repo.find_branch(&remote_rr_branch_refspec, git2::BranchType::Remote).map_err(NewIntegrateError::FindRemoteRrBranchFailed)?.get().target().ok_or(NewIntegrateError::RemoteRrBranchMissingTarget)?;

  let common_ancestor_oid = git::common_ancestor(&repo, rr_branch_oid, mainline_head_oid).map_err(NewIntegrateError::CommonAncestorFailed)?;

  let revwalk = git::get_revs(&repo, common_ancestor_oid, rr_branch_oid).map_err(NewIntegrateError::GetRevWalkerFailed)?;
  let num_of_commits = revwalk.count();

  if num_of_commits != 1 {
    return Err(NewIntegrateError::PatchBranchDoesntHaveExactlyOneCommit(remote_rr_branch_refspec, num_of_commits))
  }

  // verify that the commit in the remote request-review branch and the
  // identified patch are the same
  let rr_branch_commit = repo.find_commit(rr_branch_oid).map_err(NewIntegrateError::FindRrBranchCommitFailed)?;

  let rr_branch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &rr_branch_commit).map_err(NewIntegrateError::RrBranchCommitDiffPatchIdFailed)?;
  let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).map_err(NewIntegrateError::PatchCommitDiffPatchIdFailed)?;

  if patch_commit_diff_patch_id != rr_branch_commit_diff_patch_id {
    return Err(NewIntegrateError::PatchesDiffer)
  }

  // At this point we are pretty confident that things are properly in sync
  // and therefore we allow the actually act of integrating into to upstream
  // happen.
  let pattern = format!("refs/remotes/{}/", remote_name_str);
  let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");
  let remote_rr_branch_name = format!("{}/{}", remote_name_str, rr_branch_name);
  // e.g. git push origin origin/ps/rr/whatever-branch:main
  git::ext_push(false, remote_name_str, &remote_rr_branch_name, &upstream_branch_shorthand).map_err(NewIntegrateError::PushFailed)?;

  // Update state so that it is aware of the fact that this patch has been
  // integrated into upstream
  let short_branch_name_copy = rr_branch_name.clone();
  state_management::update_patch_state(&repo, &ps_id, |patch_meta_data_option|
    match patch_meta_data_option {
      Some(patch_meta_data) => {
        match patch_meta_data.state {
          state_management::PatchState::Published(ref _branch_name) => patch_meta_data.clone(),
          _ => {
            state_management::Patch {
              patch_id: ps_id,
              state: state_management::PatchState::Published(short_branch_name_copy)
            }
          }
        }
      },
      None => {
        state_management::Patch {
          patch_id: ps_id,
          state: state_management::PatchState::Published(short_branch_name_copy)
        }
      }
    }
  ).map_err(NewIntegrateError::UpdatePatchMetaDataFailed)?;

  // Cleanup the local and remote branches associated with this patch
  if !keep_branch {
    let mut local_branch = repo.find_branch(&rr_branch_name, git2::BranchType::Local).map_err(NewIntegrateError::DeleteLocalBranchFailed)?;
    local_branch.delete().map_err(NewIntegrateError::DeleteLocalBranchFailed)?;
    git::ext_delete_remote_branch(remote_name_str, &rr_branch_name).map_err(NewIntegrateError::DeleteRemoteBranchFailed)?;
  }

  Ok(())
}
