use super::list_config_dto::ColorWithAlternate;

#[derive(Debug)]
pub struct PsConfig {
    pub request_review: PsRequestReviewConfig,
    pub pull: PsPullConfig,
    pub integrate: PsIntegrateConfig,
    pub fetch: PsFetchConfig,
    pub list: PsListConfig,
    pub branch: PsBranchConfig,
}

#[derive(Debug)]
pub struct PsRequestReviewConfig {
    pub verify_isolation: bool,
}

#[derive(Debug)]
pub struct PsBranchConfig {
    pub verify_isolation: bool,
    pub push_to_remote: bool,
}

#[derive(Debug)]
pub struct PsPullConfig {
    pub show_list_post_pull: bool,
}

#[derive(Debug)]
pub struct PsIntegrateConfig {
    pub prompt_for_reassurance: bool,
    pub verify_isolation: bool,
    pub pull_after_integrate: bool,
}

#[derive(Debug)]
pub struct PsFetchConfig {
    pub show_upstream_patches_after_fetch: bool,
}

#[derive(Debug)]
pub struct PsListConfig {
    pub add_extra_patch_info: bool,
    pub extra_patch_info_length: usize,
    pub reverse_order: bool,
    pub alternate_colors: bool,
    pub patch_series_background: ColorWithAlternate,
    pub patch_series_foreground: ColorWithAlternate,
    pub patch_series_index: ColorWithAlternate,
    pub patch_series_sha: ColorWithAlternate,
    pub patch_series_summary: ColorWithAlternate,
    pub patch_series_extra_patch_info: ColorWithAlternate,
}
