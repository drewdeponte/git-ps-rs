use super::super::utils;
use serde::Deserialize;
use std::option::Option;
use ansi_term::Color;

#[derive(Debug, Deserialize, Clone)]
pub struct ColorWithAlternate {
    pub color: Option<Color>,
    pub color_alternate: Option<Color>
}

pub trait ColorSelector {
    fn select_color(&self, is_alternate: bool) -> Option<Color>;
}

impl ColorSelector for ColorWithAlternate {
    fn select_color(&self, is_alternate: bool) -> Option<Color> {
        // If the alternative color is not set then use the main color.
        let default = self.color.clone();
        if is_alternate {
            self.color_alternate.clone().or(default)
        } else {
            default
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ListConfigDto {
    pub add_extra_patch_info: Option<bool>,
    pub extra_patch_info_length: Option<usize>,
    pub reverse_order: Option<bool>,
    pub alternate_colors: Option<bool>,
    pub patch_series_background: Option<ColorWithAlternate>,
    pub patch_series_foreground: Option<ColorWithAlternate>,
    pub patch_series_index: Option<ColorWithAlternate>,
    pub patch_series_sha: Option<ColorWithAlternate>,
    pub patch_series_summary: Option<ColorWithAlternate>,
    pub patch_series_extra_patch_info: Option<ColorWithAlternate>,
}

impl utils::Mergable for ListConfigDto {
    /// Merge the provided b with self overriding with any present values
    fn merge(&self, b: &Self) -> Self {
        ListConfigDto {
            add_extra_patch_info: b.add_extra_patch_info.or(self.add_extra_patch_info),
            extra_patch_info_length: b.extra_patch_info_length.or(self.extra_patch_info_length),
            reverse_order: b.reverse_order.or(self.reverse_order),
            alternate_colors: b.alternate_colors.or(self.alternate_colors),
            patch_series_background: b.patch_series_background.clone().or(self.patch_series_background.clone()),
            patch_series_foreground: b.patch_series_foreground.clone().or(self.patch_series_foreground.clone()),
            patch_series_index: b.patch_series_index.clone().or(self.patch_series_index.clone()),
            patch_series_sha: b.patch_series_sha.clone().or(self.patch_series_sha.clone()),
            patch_series_summary: b.patch_series_summary.clone().or(self.patch_series_summary.clone()),
            patch_series_extra_patch_info: b.patch_series_extra_patch_info.clone().or(self.patch_series_extra_patch_info.clone()),
        }
    }
}