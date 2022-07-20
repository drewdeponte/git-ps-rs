use super::request_review_config_dto::*;
use super::pull_config_dto::*;
use super::integrate_config_dto::*;
use super::fetch_config_dto::*;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigDto {
  pub request_review: Option<RequestReviewConfigDto>,
  pub pull: Option<PullConfigDto>,
  pub integrate: Option<IntegrateConfigDto>,
  pub fetch: Option<FetchConfigDto>
}

impl utils::Mergable for ConfigDto {
  fn merge(&self, b: &Self) -> Self {
    ConfigDto {
      request_review: b.request_review.as_ref().map(|b_rr| self.request_review.as_ref().map(|a_rr| a_rr.merge(b_rr)).unwrap_or_else(|| (*b_rr).clone())).or_else(|| self.request_review.clone()),
      pull: b.pull.as_ref().map(|b_pull| self.pull.as_ref().map(|a_pull| a_pull.merge(b_pull)).unwrap_or_else(|| (*b_pull).clone())).or_else(|| self.pull.clone()),
      integrate: b.integrate.as_ref().map(|b_int| self.integrate.as_ref().map(|a_int| a_int.merge(b_int)).unwrap_or_else(|| (*b_int).clone())).or_else(|| self.integrate.clone()),
      fetch: b.fetch.as_ref().map(|b_fetch| self.fetch.as_ref().map(|a_fetch| a_fetch.merge(b_fetch)).unwrap_or_else(|| (*b_fetch).clone())).or_else(|| self.fetch.clone())
    }
  }
}
