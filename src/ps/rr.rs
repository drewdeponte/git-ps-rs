use super::git;
use super::super::ps;

pub fn rr(patch_index: usize) {
  let repo = git::create_cwd_repo().unwrap();

  // get remote name of current branch
  let cur_branch_name = git::get_current_branch(&repo).unwrap();
  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).unwrap();
  let remote_name = repo.branch_remote_name(&branch_upstream_name).unwrap();

  // create request review branch for patch
  let branch = ps::plumbing::branch::branch(&repo, patch_index).unwrap();

  // force push request review branch up to remote
  let branch_ref_name = branch.get().name().unwrap();
  git::ext_force_push(remote_name.as_str().unwrap(), branch_ref_name, branch_ref_name).unwrap();
}
