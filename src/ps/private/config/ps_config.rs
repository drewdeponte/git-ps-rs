#[derive(Debug)]
pub struct PsConfig {
  pub request_review: PsRequestReviewConfig,
  pub pull: PsPullConfig,
  pub integrate: PsIntegrateConfig,
  pub fetch: PsFetchConfig
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
  pub prompt_for_reassurance: bool,
  pub verify_isolation: bool,
  pub pull_after_integrate: bool
}

#[derive(Debug)]
pub struct PsFetchConfig {
  pub show_upstream_patches_after_fetch: bool
}
