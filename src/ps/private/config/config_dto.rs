use super::request_review_config_dto::*;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigDto {
  pub request_review: Option<RequestReviewConfigDto>
}

impl utils::Mergable for ConfigDto {
  fn merge(&self, b: &Self) -> Self {
    ConfigDto {
      request_review: b.request_review.as_ref().map(|b_rr| self.request_review.as_ref().map(|a_rr| a_rr.merge(b_rr)).unwrap_or_else(|| (*b_rr).clone())).or_else(|| self.request_review.clone())
    }
  }
}
