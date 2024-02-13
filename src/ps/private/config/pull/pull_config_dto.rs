use super::super::super::utils;
use serde::Deserialize;
use std::option::Option;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PullConfigDto {
    pub show_list_post_pull: Option<bool>,
}

impl utils::Mergable for PullConfigDto {
    /// Merge the provided b with self overriding with any present values
    fn merge(&self, b: &Self) -> Self {
        PullConfigDto {
            show_list_post_pull: b.show_list_post_pull.or(self.show_list_post_pull),
        }
    }
}
