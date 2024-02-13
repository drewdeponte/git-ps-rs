#[derive(Debug)]
pub enum SignerError {
    Signing(Box<dyn std::error::Error>),
}
