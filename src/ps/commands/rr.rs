// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr subcommand. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules such
// as the `ps::ps`, `ps::git`, or `ps::utils`.

use super::super::git;
use super::super::ps;

pub fn rr(patch_index: usize) {
  println!("patch-index: {}", patch_index);

  let repo = git::create_cwd_repo().unwrap();

  let patch_stack = ps::get_patch_stack(&repo).unwrap();
  let patches_vec = ps::get_patch_list(&repo, patch_stack);
  let patch_oid = patches_vec.get(patch_index).unwrap().oid;
  println!("patch: {}", patch_oid);

  let patch_commit = repo.find_commit(patch_oid).unwrap();
  let patch_message = patch_commit.message().unwrap();
  println!("patch message: {}", patch_message);

  if let Some(branch_name) = git::get_current_branch(&repo) {
    println!("branch-name: {}", branch_name);
  } else {
    println!("failed to get current branch name");
  }

  if let Some(ps_id) = ps::extract_ps_id(patch_message) {
    println!("patch-stack-id: {}", ps_id);
  } else {
    // add patch stack id to the commit
  }

  // - get patch given the patch index
  //    - have a map of patch index to patches
  //    - look up the reference for the given patch index
  //    - get the description from commit reference
  //    - parse ps-id out of description

  // - check if uncommited changes are present, if they are bail

  // - get the currently checked out branch
  // let head_ref = repo.head().unwrap();
  // let head_branch_shorthand = head_ref.shorthand().unwrap();
  // let head_branch_name = head_ref.name().unwrap();

  // - get the upstream branch
  // let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).unwrap();

  // - attempt to extract the id from the patch
  // - if have patch id
  //    - get associated branch name from data store
  //    - if find it in data store use it
  //    - else if not in data store generate slug branch name
  //    - create or update the request review branch
  //    - force push up to remote
  //    - record the RequestReviewRecord
  //    - checkout the originally checked out branch
  // - else if don't have patch id
  //    - add ID to patch
  //

}
