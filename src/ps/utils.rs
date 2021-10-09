// This is the `utils` module. It is responsible for housing generic utility
// functionality that this application needs. This should be functionality
// that is generic enough that other applications in theory would find it
// useful when tackeling a completely different problem space than Git Patch
// Stack. All code fitting that description belongs here.

use std::process::{Command, ExitStatus};
use std::io::Result;

/// Execute an external command in the foreground allowing it to take over the
/// terminal while waiting for the external application to complete with an
/// exit status.
pub fn execute(exe: &str, args: &[&str]) -> Result<ExitStatus> {
  Command::new(exe).args(args).spawn()?.wait()
}
