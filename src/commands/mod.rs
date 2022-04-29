// This is the `commands` module, it is the parenting module collecting all
// the other command modules. This module has two responsibility, loading
// it's respective command modules and exposing them externally. All code
// related to these responsibilities belongs here.

pub mod request_review_branch;
pub mod branch;
pub mod checkout;
pub mod isolate;
pub mod integrate;
pub mod list;
pub mod pull;
pub mod rebase;
pub mod request_review;
pub mod show;
pub mod sync;
pub mod create_patch;
pub mod amend_patch;
pub mod status;
pub mod add_changes_to_stage;
