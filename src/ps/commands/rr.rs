// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr subcommand. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules such
// as the `ps::ps`, `ps::git`, or `ps::utils`.

use super::super::utils;
use super::super::git;
use super::super::ps;
use uuid::Uuid;

pub fn rr(patch_index: usize) {
  println!("patch_index: {}", patch_index);

  let repo = git::create_cwd_repo().unwrap();

  // - find the patch identified by the patch_index
  let patch_stack = ps::get_patch_stack(&repo).unwrap();
  let patch_stack_base_commit = patch_stack.base.peel_to_commit().unwrap();
  let patches_vec = ps::get_patch_list(&repo, patch_stack);

  // TODO: add error checking for not finding a patch with the given index and
  // notify the user that they need to specify a valid index
  let patch_oid = patches_vec.get(patch_index).unwrap().oid;
  println!("patch_oid: {}", patch_oid);

  let patch_commit = repo.find_commit(patch_oid).unwrap();
  let patch_message = patch_commit.message().unwrap();

  // - if patch doesn't have patch id, add one
  let new_patch_oid: git2::Oid;
  if let Some(ps_id) = ps::extract_ps_id(patch_message) {
    new_patch_oid = patch_oid;
  } else {
    // add patch stack id to the commit
    new_patch_oid = ps::add_ps_id(&repo, patch_oid, Uuid::new_v4()).unwrap();
  }

  // - create rr branch based on upstream branch
  let patch_summary = patch_commit.summary().unwrap();
  let branch_name = ps::generate_rr_branch_name(patch_summary);
  let branch = repo.branch(branch_name.as_str(), &patch_stack_base_commit, false).unwrap();
  
  let branch_ref_name = branch.get().name().unwrap();
  println!("branch_ref_name: {}", branch_ref_name);

  // - cherry pick the patch onto new rr branch
  let cherry_picked_patch_oid = git::cherry_pick_no_working_copy(&repo, new_patch_oid, branch_ref_name).unwrap();
  println!("cherry_picked_patch_oid: {}", cherry_picked_patch_oid);

  // TODO: add pushing up to the remote
  // - push rr branch up as a remote branch

  let cur_branch_name = git::get_current_branch(&repo).unwrap();
  println!("cur_branch_name: {}", cur_branch_name);

  let branch_upstream_name = git::branch_upstream_name(&repo, cur_branch_name.as_str()).unwrap();
  println!("branch_upstream_name: {}", branch_upstream_name);

  let remote_name = repo.branch_remote_name(&branch_upstream_name).unwrap();
  println!("remote_name: {}", remote_name.as_str().unwrap());

  let refspecs = format!("{}:{}", branch_ref_name, branch_ref_name);
  println!("git push -f {} {}", remote_name.as_str().unwrap(), refspecs);
  match utils::execute("git", &["push", "-f", remote_name.as_str().unwrap(), &refspecs]) {
    Ok(_) => return,
    Err(e) => println!("error: {:?}", e)
  }
}
