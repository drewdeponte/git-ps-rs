use std::option::Option;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IntegrateConfigDto {
  pub prompt_for_reassurance: Option<bool>,
  pub verify_isolation: Option<bool>,
  pub pull_after_integrate: Option<bool>
}

impl utils::Mergable for IntegrateConfigDto {
  /// Merge the provided b with self overriding with any present values
  fn merge(&self, b: &Self) -> Self {
    IntegrateConfigDto {
      prompt_for_reassurance: b.prompt_for_reassurance.or(self.prompt_for_reassurance),
      verify_isolation: b.verify_isolation.or(self.verify_isolation),
      pull_after_integrate: b.pull_after_integrate.or(self.pull_after_integrate),
    }
  }
}
