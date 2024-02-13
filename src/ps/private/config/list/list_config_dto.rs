use super::super::super::utils;
use ansi_term::Color;
use serde::Deserialize;
use std::option::Option;

#[derive(Debug, Deserialize, Clone)]
pub struct ColorWithAlternate {
    pub color: Option<Color>,
    pub alternate_color: Option<Color>,
}

pub trait ColorSelector {
    fn select_color(&self, is_alternate: bool) -> Option<Color>;
}

impl ColorSelector for ColorWithAlternate {
    fn select_color(&self, is_alternate: bool) -> Option<Color> {
        // If the alternative color is not set then use the main color.
        let default = self.color;
        if is_alternate {
            self.alternate_color.or(default)
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
    pub alternate_patch_series_colors: Option<bool>,
    pub patch_background: Option<ColorWithAlternate>,
    pub patch_foreground: Option<ColorWithAlternate>,
    pub patch_index: Option<ColorWithAlternate>,
    pub patch_sha: Option<ColorWithAlternate>,
    pub patch_summary: Option<ColorWithAlternate>,
    pub patch_extra_info: Option<ColorWithAlternate>,
}

impl utils::Mergable for ListConfigDto {
    /// Merge the provided b with self overriding with any present values
    fn merge(&self, b: &Self) -> Self {
        ListConfigDto {
            add_extra_patch_info: b.add_extra_patch_info.or(self.add_extra_patch_info),
            extra_patch_info_length: b.extra_patch_info_length.or(self.extra_patch_info_length),
            reverse_order: b.reverse_order.or(self.reverse_order),
            alternate_patch_series_colors: b
                .alternate_patch_series_colors
                .or(self.alternate_patch_series_colors),
            patch_background: b.patch_background.clone().or(self.patch_background.clone()),
            patch_foreground: b.patch_foreground.clone().or(self.patch_foreground.clone()),
            patch_index: b.patch_index.clone().or(self.patch_index.clone()),
            patch_sha: b.patch_sha.clone().or(self.patch_sha.clone()),
            patch_summary: b.patch_summary.clone().or(self.patch_summary.clone()),
            patch_extra_info: b.patch_extra_info.clone().or(self.patch_extra_info.clone()),
        }
    }
}
