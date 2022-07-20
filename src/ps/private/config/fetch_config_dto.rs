use std::option::Option;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FetchConfigDto {
  pub show_upstream_patches_after_fetch: Option<bool>
}

impl utils::Mergable for FetchConfigDto {
  /// Merge the provided b with self overriding with any present values
  fn merge(&self, b: &Self) -> Self {
    FetchConfigDto {
      show_upstream_patches_after_fetch: b.show_upstream_patches_after_fetch.or(self.show_upstream_patches_after_fetch),
    }
  }
}
