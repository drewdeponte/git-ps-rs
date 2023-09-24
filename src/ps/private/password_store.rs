#[cfg(target_os = "macos")]
use security_framework::passwords;

#[derive(Debug)]
pub enum GetSshKeyPasswordError {
    Unknown(Box<dyn std::error::Error>),
    PasswordNotUtf8(std::string::FromUtf8Error),
}

const SSH_KEY_PASSWORD_KEYCHAIN_SERVICE: &str =
    "com.uptechworks.git-patch-stack.ssh-key.passphrases";

impl std::fmt::Display for GetSshKeyPasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetSshKeyPasswordError::Unknown(e) => write!(
                f,
                "Unknown error attempting to get SSH key password: {:?}",
                *e
            ),
            GetSshKeyPasswordError::PasswordNotUtf8(e) => {
                write!(f, "Obtained password was not utf8: {:?}", e)
            }
        }
    }
}

impl std::error::Error for GetSshKeyPasswordError {}

#[cfg(target_os = "macos")]
pub fn get_ssh_key_password(key_path: &str) -> Result<Option<String>, GetSshKeyPasswordError> {
    match passwords::get_generic_password(SSH_KEY_PASSWORD_KEYCHAIN_SERVICE, key_path) {
        Ok(pw_bytes) => String::from_utf8(pw_bytes)
            .map(Some)
            .map_err(GetSshKeyPasswordError::PasswordNotUtf8),
        Err(e) => {
            if e.code() == security_framework_sys::base::errSecItemNotFound {
                Ok(None)
            } else {
                Err(GetSshKeyPasswordError::Unknown(Box::new(e)))
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_ssh_key_password(key_path: &str) -> Result<Option<String>, GetSshKeyPasswordError> {
    Ok(None)
}

#[derive(Debug)]
pub enum SetSshKeyPasswordError {
    Unknown(Box<dyn std::error::Error>),
}

impl std::fmt::Display for SetSshKeyPasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetSshKeyPasswordError::Unknown(e) => {
                write!(f, "Unknown error setting an SSH key password: {:?}", *e)
            }
        }
    }
}

impl std::error::Error for SetSshKeyPasswordError {}

#[cfg(target_os = "macos")]
pub fn set_ssh_key_password(key_path: &str, password: &str) -> Result<(), SetSshKeyPasswordError> {
    passwords::set_generic_password(
        SSH_KEY_PASSWORD_KEYCHAIN_SERVICE,
        key_path,
        password.as_bytes(),
    )
    .map_err(|e| SetSshKeyPasswordError::Unknown(Box::new(e)))
}

#[cfg(not(target_os = "macos"))]
pub fn set_ssh_key_password(key_path: &str, password: &str) -> Result<(), SetSshKeyPasswordError> {
    Ok(())
}
