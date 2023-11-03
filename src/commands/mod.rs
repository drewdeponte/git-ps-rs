// This is the `commands` module, it is the parenting module collecting all
// the other command modules. This module has two responsibility, loading
// it's respective command modules and exposing them externally. All code
// related to these responsibilities belongs here.

pub mod backup_stack;
pub mod branch;
pub mod checkout;
pub mod fetch;
pub mod integrate;
pub mod isolate;
pub mod list;
pub mod patch_index_range;
pub mod patch_index_range_batch;
pub mod pull;
pub mod rebase;
pub mod request_review;
pub mod sha;
pub mod show;
pub mod utils;
