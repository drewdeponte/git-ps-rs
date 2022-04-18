use super::super::private::git;
use super::super::super::ps;
use super::super::private::state_management;
use uuid::Uuid;

#[derive(Debug)]
pub enum IntegrateError {
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
  DeleteRemoteBranchFailed(git::ExtDeleteRemoteBranchError),
  BranchOperationFailed(ps::private::branch::BranchError),
  GetBranchNameFailed(git2::Error),
  CreatedBranchMissingName
}

pub fn integrate(patch_index: usize, force: bool, keep_branch: bool, given_branch_name_option: Option<String>) -> Result<(), IntegrateError> {
  let repo = git::create_cwd_repo().map_err(|_| IntegrateError::RepositoryNotFound)?;

  // verify that the patch-index has a corresponding commit
  let patch_commit = ps::find_patch_commit(&repo, patch_index).map_err(IntegrateError::FindPatchCommitFailed)?;

  // verify that the commit has a patch stack id
  let ps_id = ps::commit_ps_id(&patch_commit).ok_or(IntegrateError::CommitPsIdMissing)?;

  if force {
    let (branch, ps_id) = ps::private::branch::branch(&repo, patch_index, given_branch_name_option).map_err(IntegrateError::BranchOperationFailed)?;

    // publish the patch from the local rr branch up to uptstream
    let rr_branch_name = branch.name().map_err(IntegrateError::GetBranchNameFailed)?.ok_or(IntegrateError::CreatedBranchMissingName)?;

    let cur_branch_name = git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
    let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
    let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| IntegrateError::GetRemoteNameFailed)?;
    let remote_name_str = remote_name.as_str().ok_or(IntegrateError::ConvertStringToStrFailed)?;

    let pattern = format!("refs/remotes/{}/", remote_name_str);
    let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");
    // e.g. git push origin ps/rr/whatever-branch:main
    git::ext_push(false, remote_name_str, rr_branch_name, &upstream_branch_shorthand).map_err(IntegrateError::PushFailed)?;

    // update state of the patch to indicate it has been integrated
    update_state(&repo, remote_name_str.to_string(), rr_branch_name.to_string(), ps_id)?;
    
    // clean up the local rr branch
    if !keep_branch {
      let mut local_branch = repo.find_branch(rr_branch_name, git2::BranchType::Local).map_err(IntegrateError::DeleteLocalBranchFailed)?;
      local_branch.delete().map_err(IntegrateError::DeleteLocalBranchFailed)?;
    }
  } else {
    // verify that the patch has an associated branch and has been synced
    let patch_meta_data = ps::get_patch_meta_data(&repo, ps_id).map_err(IntegrateError::GetPatchMetaDataFailed)?.ok_or(IntegrateError::PatchMetaDataMissing)?;
    if !patch_meta_data.state.has_been_pushed_to_remote() {
      return Err(IntegrateError::PatchHasNotBeenPushed)
    }

    // TODO: verify that the patch has been requested-review

    // verify remote request-review branch has exactly one commit
    let rr_branch_name = patch_meta_data.state.branch_name();

    let cur_branch_name = git::get_current_branch(&repo).ok_or(IntegrateError::CurrentBranchNameMissing)?;
    let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| IntegrateError::GetUpstreamBranchNameFailed)?;
    let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| IntegrateError::GetRemoteNameFailed)?;

    let remote_name_str = remote_name.as_str().ok_or(IntegrateError::ConvertStringToStrFailed)?;
    let mainline_head_oid = repo.head().map_err(IntegrateError::GetHeadFailed)?.target().ok_or(IntegrateError::HeadMissingTarget)?;
    let remote_rr_branch_refspec = format!("{}/{}", remote_name_str, rr_branch_name.as_str());
    let rr_branch_oid = repo.find_branch(&remote_rr_branch_refspec, git2::BranchType::Remote).map_err(IntegrateError::FindRemoteRrBranchFailed)?.get().target().ok_or(IntegrateError::RemoteRrBranchMissingTarget)?;

    let common_ancestor_oid = git::common_ancestor(&repo, rr_branch_oid, mainline_head_oid).map_err(IntegrateError::CommonAncestorFailed)?;

    let revwalk = git::get_revs(&repo, common_ancestor_oid, rr_branch_oid).map_err(IntegrateError::GetRevWalkerFailed)?;
    let num_of_commits = revwalk.count();

    if num_of_commits != 1 {
      return Err(IntegrateError::PatchBranchDoesntHaveExactlyOneCommit(remote_rr_branch_refspec, num_of_commits))
    }

    // verify that the commit in the remote request-review branch and the
    // identified patch are the same
    let rr_branch_commit = repo.find_commit(rr_branch_oid).map_err(IntegrateError::FindRrBranchCommitFailed)?;

    let rr_branch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &rr_branch_commit).map_err(IntegrateError::RrBranchCommitDiffPatchIdFailed)?;
    let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).map_err(IntegrateError::PatchCommitDiffPatchIdFailed)?;

    if patch_commit_diff_patch_id != rr_branch_commit_diff_patch_id {
      return Err(IntegrateError::PatchesDiffer)
    }

    // At this point we are pretty confident that things are properly in sync
    // and therefore we allow the actually act of integrating into to upstream
    // happen.
    let pattern = format!("refs/remotes/{}/", remote_name_str);
    let upstream_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");
    let remote_rr_branch_name = format!("{}/{}", remote_name_str, rr_branch_name);
    // e.g. git push origin origin/ps/rr/whatever-branch:main
    git::ext_push(false, remote_name_str, &remote_rr_branch_name, &upstream_branch_shorthand).map_err(IntegrateError::PushFailed)?;

    // Update state so that it is aware of the fact that this patch has been
    // integrated into upstream
    update_state(&repo, remote_name_str.to_string(), rr_branch_name.clone(), ps_id)?;

    // Cleanup the local and remote branches associated with this patch
    if !keep_branch {
      let mut local_branch = repo.find_branch(&rr_branch_name, git2::BranchType::Local).map_err(IntegrateError::DeleteLocalBranchFailed)?;
      local_branch.delete().map_err(IntegrateError::DeleteLocalBranchFailed)?;
      git::ext_delete_remote_branch(remote_name_str, &rr_branch_name).map_err(IntegrateError::DeleteRemoteBranchFailed)?;
    }
  }

  Ok(())
}

fn update_state(repo: &git2::Repository, remote_name: String, rr_branch_name: String, ps_id: Uuid) -> Result<(), IntegrateError> {
    state_management::update_patch_state(repo, &ps_id, |patch_meta_data_option|
      match patch_meta_data_option {
        Some(patch_meta_data) => {
          match patch_meta_data.state {
            state_management::PatchState::Published(_, _) => patch_meta_data.clone(),
            _ => {
              state_management::Patch {
                patch_id: ps_id,
                state: state_management::PatchState::Published(remote_name, rr_branch_name)
              }
            }
          }
        },
        None => {
          state_management::Patch {
            patch_id: ps_id,
            state: state_management::PatchState::Published(remote_name, rr_branch_name)
          }
        }
      }
    ).map_err(IntegrateError::UpdatePatchMetaDataFailed)?;
    Ok(())
}
