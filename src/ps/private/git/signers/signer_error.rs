#[derive(Debug)]
pub enum SignerError {
    #[allow(dead_code)]
    Signing(Box<dyn std::error::Error>),
}
