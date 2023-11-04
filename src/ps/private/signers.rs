use super::password_store;
use super::utils;
use ssh_key::PrivateKey;

#[derive(Debug)]
pub enum SignerError {
    KeyParsing(Box<dyn std::error::Error>),
    ReadPassword(std::io::Error),
    KeyDecryption(Box<dyn std::error::Error>),
    Signing(Box<dyn std::error::Error>),
    SignatureFormatting(Box<dyn std::error::Error>),
    GetPassword(password_store::GetSshKeyPasswordError),
    SetPassword(password_store::SetSshKeyPasswordError),
}

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

pub fn gpg_signer(
    signing_key: String,
    program: Option<String>,
) -> impl Fn(String) -> Result<String, SignerError> {
    move |commit_string: String| {
        gpg_sign_string(commit_string, signing_key.clone(), program.clone())
            .map_err(|e| SignerError::Signing(e.into()))
    }
}

#[derive(Debug)]
enum GpgSignStringError {
    Unhandled(Box<dyn std::error::Error>),
}

impl std::fmt::Display for GpgSignStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpgSignStringError::Unhandled(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GpgSignStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unhandled(e) => Some(e.as_ref()),
        }
    }
}

fn gpg_sign_string(
    commit: String,
    signing_key: String,
    program: Option<String>,
) -> Result<String, GpgSignStringError> {
    let prog = program.unwrap_or("gpg".to_string());
    let output = utils::execute_with_input_and_output(
        &commit,
        &prog,
        &[
            "--local-user",
            &signing_key,
            "--sign",
            "--armor",
            "--detach-sig",
        ],
    )
    .map_err(|e| GpgSignStringError::Unhandled(e.into()))?;

    String::from_utf8(output.stdout).map_err(|e| GpgSignStringError::Unhandled(e.into()))
}
