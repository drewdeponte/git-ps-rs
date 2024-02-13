use super::super::super::utils;
use serde::Deserialize;
use std::option::Option;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BranchConfigDto {
    pub verify_isolation: Option<bool>,
    pub push_to_remote: Option<bool>,
}

impl utils::Mergable for BranchConfigDto {
    /// Merge the provided b with self overriding with any present values
    fn merge(&self, b: &Self) -> Self {
        BranchConfigDto {
            verify_isolation: b.verify_isolation.or(self.verify_isolation),
            push_to_remote: b.push_to_remote.or(self.push_to_remote),
        }
    }
}
