use std::io::{self, Write};
#[cfg(target_family = "unix")]
use std::os::unix::process::ExitStatusExt;
#[cfg(target_family = "windows")]
use std::os::windows::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Output, Stdio};
use std::result::Result;

#[derive(Debug)]
pub enum ExecuteError {
    SpawnFailure(io::Error),
    Failure(io::Error),
    ExitStatus(i32),
    ExitSignal(i32),
    ExitMissingSignal, // triggered when we understand exit to be triggered by signal but no signal found
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpawnFailure(e) => write!(f, "Spawn failed with {}", e),
            Self::Failure(e) => write!(f, "Execute failure {}", e),
            Self::ExitStatus(status) => write!(f, "Execute exited with status {}", status),
            Self::ExitSignal(signal) => write!(f, "Execute exited with signal {}", signal),
            Self::ExitMissingSignal => write!(f, ""),
        }
    }
}

impl std::error::Error for ExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SpawnFailure(e) => Some(e),
            Self::Failure(e) => Some(e),
            Self::ExitStatus(_) => None,
            Self::ExitSignal(_) => None,
            Self::ExitMissingSignal => None,
        }
    }
}

#[cfg(target_family = "unix")]
fn handle_error_no_code(status: ExitStatus) -> ExecuteError {
    match status.signal() {
        Some(signal) => ExecuteError::ExitSignal(signal),
        None => ExecuteError::ExitMissingSignal,
    }
}

#[cfg(target_family = "windows")]
fn handle_error_no_code(_: ExitStatus) -> ExecuteError {
    return ExecuteError::ExitMissingSignal;
}

/// Execute an external command in the foreground allowing it to take over the
/// terminal while waiting for the external application to complete with an
/// exit status.
pub fn execute(exe: &str, args: &[&str]) -> Result<(), ExecuteError> {
    match Command::new(exe).args(args).spawn() {
        Err(e) => Err(ExecuteError::SpawnFailure(e)),
        Ok(mut child) => match child.wait() {
            Err(e) => Err(ExecuteError::Failure(e)),
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    Err(match status.code() {
                        Some(code) => ExecuteError::ExitStatus(code),
                        None => handle_error_no_code(status),
                    })
                }
            }
        },
    }
}

#[derive(Debug)]
pub enum ExecuteWithOutputError {
    Failure(io::Error),
}

impl std::fmt::Display for ExecuteWithOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failure(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ExecuteWithOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failure(e) => Some(e),
        }
    }
}

pub fn execute_with_output(exe: &str, args: &[&str]) -> Result<Output, ExecuteWithOutputError> {
    Command::new(exe)
        .args(args)
        .output()
        .map_err(ExecuteWithOutputError::Failure)
}

#[derive(Debug)]
pub enum ExecuteWithInputAndOutputError {
    StdinMissing,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ExecuteWithInputAndOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdinMissing => write!(f, "stdin is somehow missing for this command"),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ExecuteWithInputAndOutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::StdinMissing => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

pub fn execute_with_input_and_output(
    input: &str,
    exe: &str,
    args: &[&str],
) -> Result<Output, ExecuteWithInputAndOutputError> {
    let mut child = Command::new(exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| ExecuteWithInputAndOutputError::Unhandled(e.into()))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or(ExecuteWithInputAndOutputError::StdinMissing)?;

    let input_string = input.to_string();
    std::thread::spawn(move || {
        stdin
            .write_all(input_string.as_bytes())
            .expect("Failed to write to stdin");
    });

    child
        .wait_with_output()
        .map_err(|e| ExecuteWithInputAndOutputError::Unhandled(e.into()))
}
