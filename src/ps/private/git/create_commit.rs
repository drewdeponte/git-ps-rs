use super::config_get_bool::config_get_bool;
use super::config_get_error::ConfigGetError;
use super::config_get_string::config_get_string;
use super::create_signed_commit::{create_signed_commit, CreateSignedCommitError};
use super::create_unsigned_commit::{create_unsigned_commit, CreateUnsignedCommitError};
use super::signers;
use git2;
use std::fs;
use std::result::Result;
use std::str;

#[derive(Debug)]
pub enum CreateCommitError {
    GetCommitGpgsignFailed(ConfigGetError),
    GetGpgFormatFailed(ConfigGetError),
    GetUserSigningKeyFailed(ConfigGetError),
    CreateSignedCommitFailed(CreateSignedCommitError),
    CreateUnsignedCommitFailed(CreateUnsignedCommitError),
    ReadSshSigningKeyFailed(std::io::Error),
    UserSigningKeyNotFoundInGitConfig,
    Unhandled(Box<dyn std::error::Error>),
}

pub fn create_commit(
    repo: &'_ git2::Repository,
    config: &'_ git2::Config,
    dest_ref_name: &str,
    author: &git2::Signature<'_>,
    committer: &git2::Signature<'_>,
    message: &str,
    tree: &git2::Tree<'_>,
    parents: &[&git2::Commit<'_>],
) -> Result<git2::Oid, CreateCommitError> {
    let sign_commit_flag = config_get_bool(config, "commit.gpgsign")
        .map_err(CreateCommitError::GetCommitGpgsignFailed)?
        .unwrap_or(false);

    if sign_commit_flag {
        let gpg_format_option = config_get_string(config, "gpg.format")
            .map_err(CreateCommitError::GetGpgFormatFailed)?;

        match gpg_format_option {
            Some(val) => {
                // If program is specified for the desired format, use that program to sign the commit.
                // Otherwise, fallback to the general program (legacy for opengpg).
                let gpg_program_option = config_get_string(config, &format!("gpg.{}.program", val))
                    .and_then(|opt| {
                        opt.map_or_else(
                            || config_get_string(config, "gpg.program"),
                            |v| Ok(Some(v)),
                        )
                    })
                    .map_err(|e| CreateCommitError::Unhandled(e.into()))?;

                match val.as_str() {
                    "openpgp" => {
                        let signing_key = config_get_string(config, "user.signingkey")
                            .map_err(CreateCommitError::GetUserSigningKeyFailed)?
                            .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;

                        create_signed_commit(
                            repo,
                            signers::gpg_signer(signing_key, gpg_program_option),
                            dest_ref_name,
                            author,
                            committer,
                            message,
                            tree,
                            parents,
                        )
                        .map_err(CreateCommitError::CreateSignedCommitFailed)
                    }
                    "ssh" => {
                        let signing_key_path = config_get_string(config, "user.signingkey")
                            .map_err(CreateCommitError::GetUserSigningKeyFailed)?
                            .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;

                        let encoded_key = fs::read_to_string(&signing_key_path)
                            .map_err(CreateCommitError::ReadSshSigningKeyFailed)?;

                        create_signed_commit(
                            repo,
                            signers::ssh_signer(signing_key_path, encoded_key),
                            dest_ref_name,
                            author,
                            committer,
                            message,
                            tree,
                            parents,
                        )
                        .map_err(CreateCommitError::CreateSignedCommitFailed)
                    }
                    "x509" => {
                        eprintln!("Warning: gps currently does NOT support x509 signatures. See issue #44 - https://github.com/uptech/git-ps-rs/issues");
                        eprintln!("The commit has been created unsigned!");
                        create_unsigned_commit(
                            repo,
                            dest_ref_name,
                            author,
                            committer,
                            message,
                            tree,
                            parents,
                        )
                        .map_err(CreateCommitError::CreateUnsignedCommitFailed)
                    }
                    _ => {
                        eprintln!("Warning: gps currently only supports GPG & SSH signatures. See issue #44 - https://github.com/uptech/git-ps-rs/issues");
                        eprintln!("The commit has been created unsigned!");
                        create_unsigned_commit(
                            repo,
                            dest_ref_name,
                            author,
                            committer,
                            message,
                            tree,
                            parents,
                        )
                        .map_err(CreateCommitError::CreateUnsignedCommitFailed)
                    }
                }
            }
            None => {
                eprintln!("Warning: Your git config gpg.format doesn't appear to be set even though commit.gpgsign is enabled");
                eprintln!("The commit has been created unsigned!");
                create_unsigned_commit(
                    repo,
                    dest_ref_name,
                    author,
                    committer,
                    message,
                    tree,
                    parents,
                )
                .map_err(CreateCommitError::CreateUnsignedCommitFailed)
            }
        }
    } else {
        create_unsigned_commit(
            repo,
            dest_ref_name,
            author,
            committer,
            message,
            tree,
            parents,
        )
        .map_err(CreateCommitError::CreateUnsignedCommitFailed)
    }
}
