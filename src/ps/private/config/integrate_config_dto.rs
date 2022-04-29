use std::option::Option;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IntegrateConfigDto {
  pub require_verification: Option<bool>,
  pub verify_isolation: Option<bool>
}

impl utils::Mergable for IntegrateConfigDto {
  /// Merge the provided b with self overriding with any present values
  fn merge(&self, b: &Self) -> Self {
    IntegrateConfigDto {
      require_verification: b.require_verification.or(self.require_verification),
      verify_isolation: b.verify_isolation.or(self.verify_isolation)
    }
  }
}
