#[derive(Debug)]
pub struct PsConfig {
  pub request_review: PsRequestReviewConfig
}

#[derive(Debug)]
pub struct PsRequestReviewConfig {
  pub require_verification: bool
}
