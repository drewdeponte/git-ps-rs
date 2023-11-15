// This is the `git` module. It is responsible for housing
// functionality for interacting with git. Nothing in here should explicitly
// introduce patch stack concepts but obviously should be needed to support
// implementing the Patch Stack solutions at a higher level.
//
// Lets look at an example to make this more clear.
//
// fn get_commits(ps: PatchStack) -> Vec<Commit> // bad example
//
// The above is something that should NOT live in here because it introduces a
// concept specific to Patch Stack, in this case the `PatchStack` struct.
//
// We can still have the same functionality in here as it is mostly specific
// to git. If we simply write the function at the conceptual level of git
// instead it might look something like the following.
//
// fn get_comimts(head: Oid, base: Oid) -> Vec<Commit> // good example
//
// In the above two examples we can see that we are effectively providing
// the same functionality the but the API we are exposing at this level is
// constrained to the conceptual level of git and isn't aware of any Patch
// Stack specific concepts.
//
// This explicitly intended to NOT wrap libgit2. Instead it is designed to
// extend the functionality of libgit2. This means that it's functions will
// consume libgit2 types as well as potentially return libgit2 types.
//
// All code fitting that description belongs here.

mod branch_upstream_name;
mod commit_diff;
mod commit_diff_patch_id;
mod common_ancestor;
mod config_get_bool;
mod config_get_error;
mod config_get_string;
mod config_get_to_option;
mod create_commit;
mod create_cwd_repo;
mod create_signed_commit;
mod create_unsigned_commit;
mod ext_delete_remote_branch;
mod ext_fetch;
mod ext_push;
mod get_current_branch;
mod get_current_branch_shorthand;
mod get_revs;
mod get_summary;
mod git_error;
#[cfg(feature = "backup_cmd")]
mod hash_object_write;
mod in_rebase;
mod in_rebase_done_todos;
mod in_rebase_head_name;
mod in_rebase_onto;
mod in_rebase_todos;
mod line_to_rebase_todo;
#[cfg(feature = "backup_cmd")]
mod read_hashed_object;
mod rebase_todo;
mod signers;
mod str_to_rebase_todos;
#[cfg(test)]
mod test_utils;
mod uncommited_changes_exist;

pub use branch_upstream_name::*;
pub use commit_diff::*;
pub use commit_diff_patch_id::*;
pub use common_ancestor::*;
pub use config_get_bool::*;
pub use config_get_error::*;
pub use config_get_string::*;
pub use config_get_to_option::*;
pub use create_commit::*;
pub use create_cwd_repo::*;
pub use create_signed_commit::*;
pub use create_unsigned_commit::*;
pub use ext_delete_remote_branch::*;
pub use ext_fetch::*;
pub use ext_push::*;
pub use get_current_branch::*;
pub use get_current_branch_shorthand::*;
pub use get_revs::*;
pub use get_summary::*;
pub use git_error::*;
#[cfg(feature = "backup_cmd")]
pub use hash_object_write::*;
pub use in_rebase::*;
pub use in_rebase_done_todos::*;
pub use in_rebase_head_name::*;
pub use in_rebase_onto::*;
pub use in_rebase_todos::*;
pub use line_to_rebase_todo::*;
#[cfg(feature = "backup_cmd")]
pub use read_hashed_object::*;
pub use rebase_todo::*;
pub use signers::*;
pub use str_to_rebase_todos::*;
pub use uncommited_changes_exist::*;
