use crate::ps::private::utils;

use super::super::super::password_store;
use super::signer_error::SignerError;
use ssh_key::PrivateKey;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};
use tempfile::tempdir;

pub fn ssh_signer(
    encoded_key: String,
    signing_key_path: String,
) -> impl Fn(String) -> Result<String, SignerError> {
    move |commit_string| {
        let pk = PrivateKey::from_openssh(encoded_key.as_bytes())
            .map_err(|e| SignerError::KeyParsing(e.into()))?;
        if pk.is_encrypted() {
            let decrypted_pk = match password_store::get_ssh_key_password(&signing_key_path)
                .map_err(SignerError::GetPassword)?
            {
                Some(pw_store_password) => {
                    match pk.decrypt(pw_store_password.as_bytes()) {
                        Ok(dpk) => {
                            // proceed with decrypted key
                            dpk
                        }
                        Err(_) => {
                            // read password from user
                            let password =
                                rpassword::prompt_password("Your private SSH key password: ")
                                    .map_err(SignerError::ReadPassword)?;
                            // attempt to decrypt key
                            let dpk = pk
                                .decrypt(password.as_bytes())
                                .map_err(|e| SignerError::KeyDecryption(e.into()))?;
                            // store password in keychain
                            password_store::set_ssh_key_password(&signing_key_path, &password)
                                .map_err(SignerError::SetPassword)?;
                            // proceed with decrypted key
                            dpk
                        }
                    }
                }
                None => {
                    // read password from user
                    let password = rpassword::prompt_password("Your private SSH key password: ")
                        .map_err(SignerError::ReadPassword)?;
                    // attempt to decrypt key
                    let dpk = pk
                        .decrypt(password.as_bytes())
                        .map_err(|e| SignerError::KeyDecryption(e.into()))?;
                    // store password in keychain
                    password_store::set_ssh_key_password(&signing_key_path, &password)
                        .map_err(SignerError::SetPassword)?;
                    // proceed with decrypted key
                    dpk
                }
            };

            let ssh_sig = decrypted_pk
                .sign("git", ssh_key::HashAlg::Sha256, commit_string.as_bytes())
                .map_err(|e| SignerError::Signing(e.into()))?;
            let formatted_sig = ssh_sig
                .to_pem(ssh_key::LineEnding::LF)
                .map_err(|e| SignerError::SignatureFormatting(e.into()))?;
            Ok(formatted_sig)
        } else {
            let ssh_sig = pk
                .sign("git", ssh_key::HashAlg::Sha256, commit_string.as_bytes())
                .map_err(|e| SignerError::Signing(e.into()))?;
            let formatted_sig = ssh_sig
                .to_pem(ssh_key::LineEnding::LF)
                .map_err(|e| SignerError::SignatureFormatting(e.into()))?;
            Ok(formatted_sig)
        }
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
