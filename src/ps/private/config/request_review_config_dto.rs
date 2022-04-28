use std::option::Option;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct RequestReviewConfigDto {
  pub require_verification: Option<bool>
}

impl utils::Mergable for RequestReviewConfigDto {
  /// Merge the provided b with self overriding with any present values
  fn merge(&self, b: &Self) -> Self {
    RequestReviewConfigDto {
      require_verification: b.require_verification.or(self.require_verification)
    }
  }
}
