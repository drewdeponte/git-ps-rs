// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr subcommand. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules such
// as the `ps::ps`, `ps::git`, or `ps::utils`.

use crate::ps::ps::slugify;

use super::super::git;
use super::super::ps;
use uuid::Uuid;

pub fn rr(patch_index: usize) {
  println!("patch_index: {}", patch_index);

  let repo = git::create_cwd_repo().unwrap();

  let patch_stack = ps::get_patch_stack(&repo).unwrap();

  let patches_vec = ps::get_patch_list(&repo, patch_stack);
  let patch_oid = patches_vec.get(patch_index).unwrap().oid;
  println!("patch_oid: {}", patch_oid);

  // let patch_stack_start = patch_stack.head.target().unwrap();
  // let patch_stack_end = patch_stack.base.target().unwrap();

  // let patch_stack_base_commit = patch_stack.base.peel_to_commit().unwrap();
  // println!("base-patch-summary: {}", patch_stack_base_commit.summary().unwrap());

  // let patch_commit = repo.find_commit(patch_oid).unwrap();
  // let patch_summary = patch_commit.summary().unwrap();
  // let patch_message = patch_commit.message().unwrap();
  // println!("patch summary: {}", patch_summary);
  // println!("patch message: {}", patch_message);


  let new_patch_oid = ps::add_ps_id(&repo, patch_oid, Uuid::new_v4()).unwrap();


  // if let Some(ps_id) = ps::extract_ps_id(patch_message) {
  //   println!("patch-stack-id: {}", ps_id);
  // } else {
  //   println!("did NOT find patch-stack-id in commit message");
  //   // add patch stack id to the commit
  // }


  // let branch_name = ps::generate_rr_branch_name(patch_summary);

  // // create branch
  // let branch = repo.branch(branch_name.as_str(), &patch_stack_base_commit, false).unwrap();

  // // checkout the new branch
  // // TODO: extract this and generalize it into function in the git module
  // repo.checkout_tree(patch_stack_base_commit.as_object(), None).unwrap();
  // repo.set_head(format!("refs/heads/{}", branch_name.as_str()).as_str()).unwrap();
  
  // cherry-pick patch into new branch
  // git::cherry_pick(&repo, patch_oid);

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
