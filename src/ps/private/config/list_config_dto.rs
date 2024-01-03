use super::super::utils;
use serde::Deserialize;
use std::option::Option;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListConfigDto {
    pub add_extra_patch_info: Option<bool>,
    pub extra_patch_info_length: Option<usize>,
    pub reverse_order: Option<bool>,
    pub alternate_colors: Option<bool>,
}

impl utils::Mergable for ListConfigDto {
    /// Merge the provided b with self overriding with any present values
    fn merge(&self, b: &Self) -> Self {
        ListConfigDto {
            add_extra_patch_info: b.add_extra_patch_info.or(self.add_extra_patch_info),
            extra_patch_info_length: b.extra_patch_info_length.or(self.extra_patch_info_length),
            reverse_order: b.reverse_order.or(self.reverse_order),
            alternate_colors: b.alternate_colors.or(self.alternate_colors),
        }
    }
}
