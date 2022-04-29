#[derive(Debug)]
pub struct PsConfig {
  pub request_review: PsRequestReviewConfig,
  pub pull: PsPullConfig,
  pub integrate: PsIntegrateConfig
}

#[derive(Debug)]
pub struct PsRequestReviewConfig {
  pub verify_isolation: bool
}

#[derive(Debug)]
pub struct PsPullConfig {
  pub show_list_post_pull: bool
}

#[derive(Debug)]
pub struct PsIntegrateConfig {
  pub require_verification: bool,
  pub verify_isolation: bool
}
