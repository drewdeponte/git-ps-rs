use super::config::{config_get_bool, config_get_string, ConfigGetError};
use super::create_signed_commit::{create_signed_commit, CreateSignedCommitError};
use super::create_unsigned_commit::{create_unsigned_commit, CreateUnsignedCommitError};
use super::signers;
use git2;
use std::result::Result;
use std::str;

#[derive(Debug)]
pub enum CreateCommitError {
    #[allow(dead_code)]
    GetCommitGpgsignFailed(ConfigGetError),
    #[allow(dead_code)]
    GetGpgFormatFailed(ConfigGetError),
    #[allow(dead_code)]
    GetUserSigningKeyFailed(ConfigGetError),
    #[allow(dead_code)]
    CreateSignedCommitFailed(CreateSignedCommitError),
    #[allow(dead_code)]
    CreateUnsignedCommitFailed(CreateUnsignedCommitError),
    #[allow(dead_code)]
    UserSigningKeyNotFoundInGitConfig,
    #[allow(dead_code)]
    Unhandled(Box<dyn std::error::Error>),
}

#[allow(clippy::too_many_arguments)]
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
        let gpg_format = config_get_string(config, "gpg.format")
            .map_err(CreateCommitError::GetGpgFormatFailed)?
            .unwrap_or(("openpgp").to_string());

        // If program is specified for the desired format, use that program to sign the commit.
        // Otherwise, fallback to the general program (legacy for opengpg).
        let gpg_program_option = config_get_string(config, &format!("gpg.{}.program", gpg_format))
            .and_then(|opt| {
                opt.map_or_else(|| config_get_string(config, "gpg.program"), |v| Ok(Some(v)))
            })
            .map_err(|e| CreateCommitError::Unhandled(e.into()))?;

        let signing_key_config = config_get_string(config, "user.signingkey")
            .map_err(CreateCommitError::GetUserSigningKeyFailed)?
            .ok_or(CreateCommitError::UserSigningKeyNotFoundInGitConfig)?;

        match gpg_format.as_str() {
            "openpgp" => create_signed_commit(
                repo,
                signers::gpg_signer(signing_key_config, gpg_program_option),
                dest_ref_name,
                author,
                committer,
                message,
                tree,
                parents,
            )
            .map_err(CreateCommitError::CreateSignedCommitFailed),
            "ssh" => create_signed_commit(
                repo,
                signers::ssh_signer(signing_key_config, gpg_program_option),
                dest_ref_name,
                author,
                committer,
                message,
                tree,
                parents,
            )
            .map_err(CreateCommitError::CreateSignedCommitFailed),
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
