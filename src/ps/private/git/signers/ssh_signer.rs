use super::signer_error::SignerError;
use crate::ps::private::utils;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};
use tempfile::tempdir;

pub fn ssh_signer(
    signing_key: String,
    program: Option<String>,
) -> impl Fn(String) -> Result<String, SignerError> {
    move |commit_string: String| {
        ssh_sign_string(commit_string, signing_key.clone(), program.clone())
            .map_err(|e| SignerError::Signing(e.into()))
    }
}

#[derive(Debug)]
enum SshSignStringError {
    CreateTempDirFailed(io::Error),
    CreateTempFileFailed(io::Error),
    WriteTempFileFailed(io::Error),
    TempPathToStrFailed,
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for SshSignStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshSignStringError::CreateTempDirFailed(e) => write!(f, "{}", e),
            SshSignStringError::CreateTempFileFailed(e) => write!(f, "{}", e),
            SshSignStringError::WriteTempFileFailed(e) => write!(f, "{}", e),
            SshSignStringError::TempPathToStrFailed => {
                write!(f, "Failed to convert temp path to string")
            }
            SshSignStringError::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for SshSignStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CreateTempDirFailed(e) => Some(e),
            Self::CreateTempFileFailed(e) => Some(e),
            Self::WriteTempFileFailed(e) => Some(e),
            Self::TempPathToStrFailed => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

fn ssh_sign_string(
    commit: String,
    signing_key: String,
    program: Option<String>,
) -> Result<String, SshSignStringError> {
    let prog = program.unwrap_or("ssh-keygen".to_string());
    let dir = tempdir().map_err(SshSignStringError::CreateTempDirFailed)?;
    // keep the binding alive so the path doesn't get dropped
    let dir_binding = dir.path().join(".tmp_signing_key");
    let path = signing_key_path(&dir_binding, &signing_key)?;
    let output = utils::execute_with_input_and_output(
        &commit,
        &prog,
        &["-Y", "sign", "-n", "git", "-f", path],
    )
    .map_err(|e| SshSignStringError::Unhandled(e.into()))?;

    String::from_utf8(output.stdout).map_err(|e| SshSignStringError::Unhandled(e.into()))
}

fn literal_ssh_key(signing_key_config: &str) -> Option<&str> {
    if signing_key_config.starts_with("ssh-") {
        Some(signing_key_config)
    } else if let Some(stripped) = signing_key_config.strip_prefix("key::") {
        Some(stripped)
    } else {
        None
    }
}

// If the signing key is a literal SSH key, write it to a temporary file and return the path.
fn signing_key_path<'a>(
    path: &'a PathBuf,
    signing_key_config: &'a str,
) -> Result<&'a str, SshSignStringError> {
    match literal_ssh_key(signing_key_config) {
        Some(literal) => {
            let mut file = File::create(path).map_err(SshSignStringError::CreateTempFileFailed)?;
            writeln!(file, "{}", literal).map_err(SshSignStringError::WriteTempFileFailed)?;
            path.to_str().ok_or(SshSignStringError::TempPathToStrFailed)
        }
        None => Ok(signing_key_config),
    }
}
