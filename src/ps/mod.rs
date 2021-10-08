// This is the `ps` module, it is the parenting module collecting all the
// other child Patch Stack specific modules. This module has two
// responsibility, loading it's respective child modules and exposing them
// externally. All code related to these responsibilities belongs here.

pub mod ps;
pub mod git;
pub mod utils;
pub mod commands;

#[cfg(test)]
mod test;
