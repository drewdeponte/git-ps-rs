use super::super::private::git;
use super::super::super::ps;
use super::super::private::state_management;
use uuid::Uuid;

#[derive(Debug)]
pub enum NewIntegrateError {
  RepositoryNotFound,
  CurrentBranchNameMissing,
  GetUpstreamBranchNameFailed,
  GetRemoteBranchNameFailed,
  PatchBranchDoesntHaveExactlyOneCommit(String, usize) // (branch_name, num_of_commits)
  // CreateRrBranchFailed(ps::private::branch::BranchError),
  // RequestReviewBranchNameMissing,
  // ForcePushFailed(ps::private::git::ExtForcePushError),
  // PushFailed(ps::private::git::ExtForcePushError),
  // GetShortBranchNameFailed,
  // ConvertStringToStrFailed,
  // UpdatePatchMetaDataFailed(state_management::StorePatchStateError),
  // DeleteRemoteBranchFailed(git::ExtDeleteRemoteBranchError),
  // DeleteLocalBranchFailed(git2::Error)
}

pub fn new_integrate(patch_index: usize, keep_branch: bool, given_branch_name: Option<String>) -> Result<(), NewIntegrateError> {
  let repo = git::create_cwd_repo().map_err(|_| NewIntegrateError::RepositoryNotFound)?;

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).ok_or(NewIntegrateError::CurrentBranchNameMissing)?;
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).map_err(|_| NewIntegrateError::GetUpstreamBranchNameFailed)?;
  let remote_name = repo.branch_remote_name(&branch_upstream_name).map_err(|_| NewIntegrateError::GetRemoteBranchNameFailed)?;

  // create request review branch for patch
  // let (branch, ps_id) = ps::private::branch::branch(&repo, patch_index, given_branch_name).map_err(NewIntegrateError::CreateRrBranchFailed)?;

  // force push request review branch up to remote
  // let branch_ref_name = branch.get().name().ok_or(NewIntegrateError::RequestReviewBranchNameMissing)?;
  // let short_branch_name = branch.get().shorthand().ok_or(NewIntegrateError::GetShortBranchNameFailed)?.to_string();
  // git::ext_push(true, remote_name.as_str().ok_or(NewIntegrateError::ConvertStringToStrFailed)?, branch_ref_name, branch_ref_name).map_err(NewIntegrateError::ForcePushFailed)?;

  // ps/rr/my-patch-branch - this was at one point based on origin/main
  // orign/ps/rr/my-patch-branch - this is what gets updated when we do a fetch operation
  // main - this is the patch stack itself
  // origin/main - this gets updated when do a fetch operation

  // find the patch identified by the patch_index
  let patch_commit = ps::find_patch_commit(&repo, patch_index).unwrap();

  // get the diff patch id of the patch's commit
  let patch_commit_diff_patch_id = git::commit_diff_patch_id(&repo, &patch_commit).unwrap();

  // extract patch stack identifier from commit
  let ps_id = ps::commit_ps_id(&patch_commit).unwrap();

  // fetch patch's associated branch name from state
  let patch_meta_data = ps::get_patch_meta_data(&repo, ps_id).unwrap();
  let patch_branch_name = patch_meta_data.map(|pmd| pmd.state.branch_name()).unwrap();

  // fetch
  
  // get merge base between origin/main and origin/ps/rr/my-patch-branch
  // get the oid of origin/main
  let mainline_head_oid = repo.head().unwrap().target().unwrap();
  // get the oid of origin/ps/rr/my-patch-branch
  let remote_rr_branch_refspec = format!("{}/{}", remote_name.as_str().unwrap(), patch_branch_name.as_str());
  let rr_branch_oid = repo.find_branch(&remote_rr_branch_refspec, git2::BranchType::Remote).unwrap().get().target().unwrap();
  let merge_base_oid = repo.merge_base(rr_branch_oid, mainline_head_oid).unwrap();

  println!("merge_base_oid = {}", merge_base_oid);

  let merge_base_commit = repo.find_commit(merge_base_oid).unwrap();
  let common_ancestor_oid;
  if merge_base_commit.parent_count() > 0 {
    let common_ancestor_commit = merge_base_commit.parent(0).unwrap();
    common_ancestor_oid = common_ancestor_commit.id();
  } else {
    common_ancestor_oid = merge_base_commit.id();
  }

  // let common_ancestor_commit = merge_base_commit.parent(0).unwrap();
  // let common_ancestor_oid = common_ancestor_commit.id();


  let revwalk = git::get_revs(&repo, common_ancestor_oid, rr_branch_oid).unwrap();
  let num_of_commits = revwalk.count();

  // make sure it has exactly one commit in the branch
  if num_of_commits != 1 {
    return Err(NewIntegrateError::PatchBranchDoesntHaveExactlyOneCommit(remote_rr_branch_refspec, num_of_commits))
  }


  let patch_diff = git::commit_diff(&repo, &patch_commit).unwrap();
  let patch_diff_stable_id = patch_diff.patchid(Option::None).unwrap();

  println!("patch_diff_stable_id\n{}", patch_diff_stable_id);

  let rr_branch_commit = repo.find_commit(rr_branch_oid).unwrap();
  let rr_branch_commit_diff = git::commit_diff(&repo, &rr_branch_commit).unwrap();
  let rr_branch_commit_diff_stable_id = rr_branch_commit_diff.patchid(Option::None).unwrap();

  println!("rr_branch_commit_diff_stable_id\n{}", rr_branch_commit_diff_stable_id);

  if (patch_diff_stable_id != rr_branch_commit_diff_stable_id) {
    println!("WE HAVE A PROBLEM");
  }





  // make sure that the commit that remote request-review branch has contains
  // the same content that the patch in the patch stack does






  // verify that the remote request-review branch (e.g.
  // origin/ps/rr/my-patch-branch) has only one commit, that the commit it has
  // the same patch identifier, and that the contents of the patch match the
  // contents of the reference patch in the patch stack.
  //
  // verify origin/ps/rr/my-patch-branch matches local ps/rr/my-patch-branch
  //
  // if any of those aren't true, it errors out to the user and stops the
  // integration

  // - push rr branch up to upstream branch (e.g. origin/main)
  // let pattern = format!("refs/remotes/{}/", remote_name.as_str().ok_or(NewIntegrateError::ConvertStringToStrFailed)?);
  // let remote_branch_shorthand = str::replace(&branch_upstream_name, pattern.as_str(), "");
  // git::ext_push(false, remote_name.as_str().ok_or(NewIntegrateError::ConvertStringToStrFailed)?, branch_ref_name, &remote_branch_shorthand).map_err(NewIntegrateError::PushFailed)?;

  // git push origin ps/rr/my-patch-branch:main
  // git push origin origin/ps/rr/my-patch-branch:main - this is the more
  // correct thing todo

  // let short_branch_name_copy = short_branch_name.clone();
  // // associate the patch to the branch that was created
  // state_management::update_patch_state(&repo, &ps_id, |patch_meta_data_option|
  //   match patch_meta_data_option {
  //     Some(patch_meta_data) => {
  //       match patch_meta_data.state {
  //         state_management::PatchState::Published(ref _branch_name) => patch_meta_data.clone(),
  //         _ => {
  //           state_management::Patch {
  //             patch_id: ps_id,
  //             state: state_management::PatchState::Published(short_branch_name_copy)
  //           }
  //         }
  //       }
  //     },
  //     None => {
  //       state_management::Patch {
  //         patch_id: ps_id,
  //         state: state_management::PatchState::Published(short_branch_name_copy)
  //       }
  //     }
  //   }
  // ).map_err(NewIntegrateError::UpdatePatchMetaDataFailed)?;

  // if !keep_branch {
  //   let mut local_branch = repo.find_branch(&short_branch_name, git2::BranchType::Local).map_err(NewIntegrateError::DeleteLocalBranchFailed)?;
  //   local_branch.delete().map_err(NewIntegrateError::DeleteLocalBranchFailed)?;
  //   git::ext_delete_remote_branch(remote_name.as_str().ok_or(NewIntegrateError::ConvertStringToStrFailed)?, &short_branch_name).map_err(NewIntegrateError::DeleteRemoteBranchFailed)?;
  // }

  Ok(())
}
