use std::option::Option;
use serde::Deserialize;
use super::super::utils;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListConfigDto {
  pub add_extra_patch_info: Option<bool>,
  pub extra_patch_info_length: Option<usize>
}

impl utils::Mergable for ListConfigDto {
  /// Merge the provided b with self overriding with any present values
  fn merge(&self, b: &Self) -> Self {
    ListConfigDto {
      add_extra_patch_info: b.add_extra_patch_info.or(self.add_extra_patch_info),
      extra_patch_info_length: b.extra_patch_info_length.or(self.extra_patch_info_length),
    }
  }
}
