use super::super::super::password_store;

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
