// This is the `commands` module, it is the parenting module collecting all
// the other command modules. This module has two responsibility, loading
// it's respective command modules and exposing them externally. All code
// related to these responsibilities belongs here.

pub mod porcelain;
pub mod plumbing;
