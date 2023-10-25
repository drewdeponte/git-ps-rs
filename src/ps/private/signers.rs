use super::password_store;
use gpgme;
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

pub fn gpg_signer(signing_key: String) -> impl Fn(String) -> Result<String, SignerError> {
    move |commit_string: String| {
        gpg_sign_string(commit_string, signing_key.clone())
            .map_err(|e| SignerError::Signing(e.into()))
    }
}

#[derive(Debug)]
enum GpgSignStringError {
    GetGpgContext,
    GetSecretKey,
    AddSigner,
    CreateDetachedSignature,
    FromUtf8(std::string::FromUtf8Error),
}

impl std::fmt::Display for GpgSignStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpgSignStringError::GetGpgContext => write!(f, "failed to get GPG context"),
            GpgSignStringError::GetSecretKey => write!(f, "failed to get GPG secret key"),
            GpgSignStringError::AddSigner => write!(f, "failed to add signer"),
            GpgSignStringError::CreateDetachedSignature => {
                write!(f, "failed to create detached signature")
            }
            GpgSignStringError::FromUtf8(e) => {
                write!(f, "failed to interpret signature as utf8 - {}", e)
            }
        }
    }
}

impl std::error::Error for GpgSignStringError {}

fn gpg_sign_string(commit: String, signing_key: String) -> Result<String, GpgSignStringError> {
    let mut ctx = gpgme::Context::from_protocol(gpgme::Protocol::OpenPgp)
        .map_err(|_| GpgSignStringError::GetGpgContext)?;
    ctx.set_armor(true);
    let key = ctx
        .get_secret_key(signing_key)
        .map_err(|_| GpgSignStringError::GetSecretKey)?;

    ctx.add_signer(&key)
        .map_err(|_| GpgSignStringError::AddSigner)?;
    let mut output = Vec::new();
    ctx.sign_detached(commit, &mut output)
        .map_err(|_| GpgSignStringError::CreateDetachedSignature)?;

    String::from_utf8(output)
        .map(|s| s.trim().to_string())
        .map_err(GpgSignStringError::FromUtf8)
}
