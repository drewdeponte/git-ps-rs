// This is the `porcelain` module, it is the parenting module collecting all
// the other porcelain command modules. This module has two responsibility,
// loading it's respective command modules and exposing them externally. All
// code related to these responsibilities belongs here.

pub mod pull;
pub mod rr;
pub mod ls;
pub mod rebase;
pub mod integrate;
pub mod show;
pub mod checkout;
pub mod isolate;
