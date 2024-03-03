use super::signer_error::SignerError;
use crate::ps::private::utils;
use std::{fs::File, io::Write};
use tempfile::{tempdir, NamedTempFile};

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
    MissingOption,
    SignCommandFailed(Vec<u8>),
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for SshSignStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingOption => write!(f, "unexpected option missing"),
            Self::SignCommandFailed(stderr) => write!(f, "{:?}", stderr),
            Self::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for SshSignStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingOption => None,
            Self::SignCommandFailed(_) => None,
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

fn ssh_sign_string(
    string: String,
    signing_key: String,
    program: Option<String>,
) -> Result<String, SshSignStringError> {
    let prog = program.unwrap_or("ssh-keygen".to_string());
    let mut is_literal_key = false;

    let ssh_key_path = match literal_ssh_key(&signing_key) {
        Some(ssh_key_content) => {
            is_literal_key = true;

            // create a temporary directory & possibly a temporary file to hold the sigining key
            // for use with the ssh-keygen command
            let dir = tempdir().map_err(|e| SshSignStringError::Unhandled(e.into()))?;
            let tmp_ssh_key_path = dir.path().join(".tmp_signing_key");
            let mut file = File::create(tmp_ssh_key_path.as_path())
                .map_err(|e| SshSignStringError::Unhandled(e.into()))?;
            file.write(ssh_key_content.as_bytes())
                .map_err(|e| SshSignStringError::Unhandled(e.into()))?;
            tmp_ssh_key_path
        }
        None => {
            let mut path_buf = std::path::PathBuf::new();
            path_buf.push(&signing_key);
            path_buf
        }
    };

    let ssh_key_path_str = ssh_key_path
        .to_str()
        .ok_or(SshSignStringError::MissingOption)?;

    // write the string to sign to a temporary file that we can
    // reference in the ssh-keygen command
    let mut tmp_string_file =
        NamedTempFile::new().map_err(|e| SshSignStringError::Unhandled(e.into()))?;
    tmp_string_file
        .write(string.as_bytes())
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?;
    // Close the file, but keep the path to it around.
    let tmp_string_file_path = tmp_string_file.into_temp_path();
    let tmp_string_file_path_str = tmp_string_file_path
        .to_str()
        .ok_or(SshSignStringError::MissingOption)?;

    let output = match is_literal_key {
        true => utils::execute_with_output(
            &prog,
            &[
                "-Y",
                "sign",
                "-n",
                "git",
                "-f",
                ssh_key_path_str,
                "-U",
                tmp_string_file_path_str,
            ],
        )
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?,
        false => utils::execute_with_output(
            &prog,
            &[
                "-Y",
                "sign",
                "-n",
                "git",
                "-f",
                ssh_key_path_str,
                tmp_string_file_path_str,
            ],
        )
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?,
    };

    let content_to_sign_file_path = tmp_string_file_path.to_path_buf();

    tmp_string_file_path
        .close()
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?;

    if !output.status.success() {
        return Err(SshSignStringError::SignCommandFailed(output.stderr));
    }

    // read the signature from the produced file
    let content_to_sign_file_path_str = content_to_sign_file_path
        .to_str()
        .ok_or(SshSignStringError::MissingOption)?;
    let signed_content_file_path = format!("{}.sig", &content_to_sign_file_path_str);
    let signed_content = std::fs::read_to_string(&signed_content_file_path)
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?;
    std::fs::remove_file(&signed_content_file_path)
        .map_err(|e| SshSignStringError::Unhandled(e.into()))?;

    Ok(signed_content)
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
