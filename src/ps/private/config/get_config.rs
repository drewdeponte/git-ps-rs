use super::super::paths;
use super::read_config_or_default::*;
use super::config_dto::ConfigDto;
use super::pull_config_dto::PullConfigDto;
use super::request_review_config_dto::RequestReviewConfigDto;
use super::integrate_config_dto::IntegrateConfigDto;
use super::fetch_config_dto::FetchConfigDto;
use super::list_config_dto::ListConfigDto;
use super::ps_config::{PsConfig, PsPullConfig, PsRequestReviewConfig, PsIntegrateConfig, PsFetchConfig, PsListConfig};
use super::super::utils::*;

#[derive(Debug)]
pub enum GetConfigError {
  FailedToGetUserLevelConfigPath(paths::UserLevelConfigPathError),
  ReadConfigFailed(ReadConfigDtoOrDefaultError)
}

pub fn get_config(repo_root: &str) -> Result<PsConfig, GetConfigError> {
  // get the three different configs or their defaults
  let user_level_config_path = paths::user_level_config_path()
    .map_err(GetConfigError::FailedToGetUserLevelConfigPath)?;
  let user_level_config = read_config_dto_or_default(&user_level_config_path)
    .map_err(GetConfigError::ReadConfigFailed)?;

  let personal_repo_level_config_path = paths::personal_repository_level_config_path(repo_root);
  let personal_repo_config = read_config_dto_or_default(&personal_repo_level_config_path)
    .map_err(GetConfigError::ReadConfigFailed)?;

  let communal_repo_level_config_path = paths::communal_repository_level_config_path(repo_root);
  let communal_repo_config = read_config_dto_or_default(&communal_repo_level_config_path)
    .map_err(GetConfigError::ReadConfigFailed)?;

  // merge the three configs appropriately into the final config
  let config_dto = user_level_config.merge(&personal_repo_config).merge(&communal_repo_config);

  Ok(apply_config_defaults(&config_dto))
}

fn apply_config_defaults(config_dto: &ConfigDto) -> PsConfig {
  let default_rr_config = apply_request_review_config_defaults(&RequestReviewConfigDto::default());
  let default_pull_config = apply_pull_config_defaults(&PullConfigDto::default());
  let default_integrate_config = apply_integrate_config_defaults(&IntegrateConfigDto::default());
  let default_fetch_config = apply_fetch_config_defaults(&FetchConfigDto::default());
  let default_list_config = apply_list_config_defaults(&ListConfigDto::default());
  PsConfig {
    request_review: config_dto.request_review.as_ref().map(apply_request_review_config_defaults).unwrap_or(default_rr_config),
    pull: config_dto.pull.as_ref().map(apply_pull_config_defaults).unwrap_or(default_pull_config),
    integrate: config_dto.integrate.as_ref().map(apply_integrate_config_defaults).unwrap_or(default_integrate_config),
    fetch: config_dto.fetch.as_ref().map(apply_fetch_config_defaults).unwrap_or(default_fetch_config),
    list: config_dto.list.as_ref().map(apply_list_config_defaults).unwrap_or(default_list_config)
  }
}

fn apply_request_review_config_defaults(rr_config_dto: &RequestReviewConfigDto) -> PsRequestReviewConfig {
  PsRequestReviewConfig {
    verify_isolation: rr_config_dto.verify_isolation.unwrap_or(true)
  }
}

fn apply_pull_config_defaults(pull_config_dto: &PullConfigDto) -> PsPullConfig {
  PsPullConfig {
    show_list_post_pull: pull_config_dto.show_list_post_pull.unwrap_or(false)
  }
}

fn apply_integrate_config_defaults(integrate_config_dto: &IntegrateConfigDto) -> PsIntegrateConfig {
  PsIntegrateConfig {
    prompt_for_reassurance: integrate_config_dto.prompt_for_reassurance.unwrap_or(true),
    verify_isolation: integrate_config_dto.verify_isolation.unwrap_or(true),
    pull_after_integrate: integrate_config_dto.pull_after_integrate.unwrap_or(false)
  }
}

fn apply_fetch_config_defaults(fetch_config_dto: &FetchConfigDto) -> PsFetchConfig {
  PsFetchConfig {
    show_upstream_patches_after_fetch: fetch_config_dto.show_upstream_patches_after_fetch.unwrap_or(true)
  }
}

fn apply_list_config_defaults(list_config_dto: &ListConfigDto) -> PsListConfig {
  PsListConfig {
    add_extra_patch_info: list_config_dto.add_extra_patch_info.unwrap_or(false),
    extra_patch_info_length: list_config_dto.extra_patch_info_length.unwrap_or(10),
    reverse_order: list_config_dto.reverse_order.unwrap_or(false)
  }
}
