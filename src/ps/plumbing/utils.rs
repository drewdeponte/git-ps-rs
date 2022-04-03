// This is the `utils` module. It is responsible for housing generic utility
// functionality that this application needs. This should be functionality
// that is generic enough that other applications in theory would find it
// useful when tackeling a completely different problem space than Git Patch
// Stack. All code fitting that description belongs here.

use std::os::unix::prelude::ExitStatusExt;
use std::process::Command;
use std::io;
use std::result::Result;

#[derive(Debug)]
pub enum ExecuteError {
  SpawnFailure(io::Error),
  Failure(io::Error),
  ExitStatus(i32),
  ExitSignal(i32),
  ExitMissingSignal // triggered when we understand exit to be triggered by signal but no signal found
}

/// Execute an external command in the foreground allowing it to take over the
/// terminal while waiting for the external application to complete with an
/// exit status.
pub fn execute(exe: &str, args: &[&str]) -> Result<(), ExecuteError> {
  match Command::new(exe).args(args).spawn() {
    Err(e) => return Err(ExecuteError::SpawnFailure(e)),
    Ok(mut child) => match child.wait() {
      Err(e) => return Err(ExecuteError::Failure(e)),
      Ok(status) => {
        if status.success() {
          return Ok(())
        } else {
          match status.code() {
            Some(code) => return Err(ExecuteError::ExitStatus(code)),
            None => match status.signal() {
              Some(signal) => return Err(ExecuteError::ExitSignal(signal)),
              None => return Err(ExecuteError::ExitMissingSignal)
            }
          }
        }
      }
    }
  };
}
