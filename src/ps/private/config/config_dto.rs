use super::super::utils;
use super::fetch_config_dto::*;
use super::integrate_config_dto::*;
use super::list_config_dto::*;
use super::pull_config_dto::*;
use super::request_review_config_dto::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ConfigDto {
    pub request_review: Option<RequestReviewConfigDto>,
    pub pull: Option<PullConfigDto>,
    pub integrate: Option<IntegrateConfigDto>,
    pub fetch: Option<FetchConfigDto>,
    pub list: Option<ListConfigDto>,
}

impl utils::Mergable for ConfigDto {
    fn merge(&self, b: &Self) -> Self {
        ConfigDto {
            request_review: utils::merge_option(&self.request_review, &b.request_review),
            pull: utils::merge_option(&self.pull, &b.pull),
            integrate: utils::merge_option(&self.integrate, &b.integrate),
            fetch: utils::merge_option(&self.fetch, &b.fetch),
            list: utils::merge_option(&self.list, &b.list),
        }
    }
}
