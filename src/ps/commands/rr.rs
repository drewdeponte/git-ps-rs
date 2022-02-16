// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr subcommand. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules such
// as the `ps::ps`, `ps::git`, or `ps::utils`.

use crate::ps::ps::slugify;

use super::super::git;
use super::super::ps;

pub fn rr(patch_index: usize) {
  println!("patch-index: {}", patch_index);

  let repo = git::create_cwd_repo().unwrap();

  let patch_stack = ps::get_patch_stack(&repo).unwrap();
  // let patch_stack_start = patch_stack.head.target().unwrap();
  // let patch_stack_end = patch_stack.base.target().unwrap();

  // let patch_stack_base_commit = patch_stack.base.peel_to_commit().unwrap();
  // println!("base-patch-summary: {}", patch_stack_base_commit.summary().unwrap());

  let patches_vec = ps::get_patch_list(&repo, patch_stack);
  let patch_oid = patches_vec.get(patch_index).unwrap().oid;
  println!("patch: {}", patch_oid);

  // let patch_commit = repo.find_commit(patch_oid).unwrap();
  // let patch_summary = patch_commit.summary().unwrap();
  // let patch_message = patch_commit.message().unwrap();
  // println!("patch summary: {}", patch_summary);
  // println!("patch message: {}", patch_message);

  // Get currently checked out branch
  let branch_ref_name = git::get_current_branch(&repo).unwrap();
  println!("branch-name: {}",  branch_ref_name);

  // Get current branches upstream tracking branch
  let upstream_branch_ref_name = git::branch_upstream_name(&repo, &branch_ref_name).unwrap();
  println!("upstream-branch-name: {}", upstream_branch_ref_name);

  let cur_branch_obj = repo.revparse_single(&branch_ref_name).unwrap();
  let upstream_branch_obj = repo.revparse_single(&upstream_branch_ref_name).unwrap();
  let upstream_branch_commit = repo.find_commit(upstream_branch_obj.id()).unwrap();
  println!("cur_branch oid: {}", cur_branch_obj.id());
  println!("upstream_branch oid: {}", upstream_branch_obj.id());

  // Get the common ancestor
  // let common_ancestor_oid = repo.merge_base(patch_oid, upstream_branch_obj.id()).unwrap();
  // println!("common_ancestor_oid: {}", common_ancestor_oid);
  // let common_ancestor_commit = repo.find_commit(common_ancestor_oid).unwrap();

  // create branch
  let add_id_rework_branch = repo.branch("ps/tmp/add_id_rework", &upstream_branch_commit, false).unwrap();
  let add_id_rework_branch_ref_name = add_id_rework_branch.get().name().unwrap();
  println!("foo: {}", add_id_rework_branch_ref_name);
      
  // checkout the new branch
  // repo.checkout_tree(add_id_rework_branch.get().peel_to_commit().unwrap().as_object(), None).unwrap();
  // // repo.checkout_tree(common_ancestor_commit.as_object(), None).unwrap();
  // repo.set_head(add_id_rework_branch_ref_name).unwrap();

  // cherry pick
  let foo_result = git::cherry_pick_no_working_copy(&repo, patch_oid, add_id_rework_branch).unwrap();
  // git::cherry_pick(&repo, patch_oid).unwrap();


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
