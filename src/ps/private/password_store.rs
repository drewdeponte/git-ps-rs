use keyring::{Entry, Result};

#[derive(Debug)]
pub enum GetSshKeyPasswordError {
    Unknown(Box<dyn std::error::Error>),
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
        }
    }
}

impl std::error::Error for GetSshKeyPasswordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unknown(e) => Some(e.as_ref()),
        }
    }
}

fn keyring_entry(key_path: &str) -> Result<Entry> {
    Entry::new(SSH_KEY_PASSWORD_KEYCHAIN_SERVICE, key_path)
}

pub fn get_ssh_key_password(
    key_path: &str,
) -> std::result::Result<Option<String>, GetSshKeyPasswordError> {
    match keyring_entry(key_path) {
        Ok(entry) => match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(GetSshKeyPasswordError::Unknown(e.into())),
        },
        Err(keyring::error::Error::NoEntry) => Ok(None),
        Err(e) => Err(GetSshKeyPasswordError::Unknown(e.into())),
    }
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

impl std::error::Error for SetSshKeyPasswordError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unknown(e) => Some(e.as_ref()),
        }
    }
}

pub fn set_ssh_key_password(
    key_path: &str,
    password: &str,
) -> std::result::Result<(), SetSshKeyPasswordError> {
    match keyring_entry(key_path) {
        Ok(entry) => entry
            .set_password(password)
            .map_err(|e| SetSshKeyPasswordError::Unknown(e.into())),
        Err(e) => Err(SetSshKeyPasswordError::Unknown(e.into())),
    }
}
