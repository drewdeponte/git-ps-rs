use std::option::Option;
use std::string::String;

#[derive(Debug, Clone)]
pub struct PatchIndexRange {
    pub start_index: usize,
    pub end_index: Option<usize>,
}

#[derive(Debug)]
pub enum ParsePatchIndexOrRangeError {
    InvalidIndexRange(String),
    UnparsableIndex(String, std::num::ParseIntError),
    StartPatchIndexLargerThanEnd(String),
}

impl std::str::FromStr for PatchIndexRange {
    type Err = ParsePatchIndexOrRangeError;

    /// Parse string representation of an a patch index or patch index range
    ///
    /// A patch index style would simply be a string that is the patch index, e.g. "12". A patch index
    /// range on the other hand is two patch index strings separated by a dash character, e.g. "2-4".
    /// The left most patch index is the starting patch index and the right most patch index is the
    /// ending patch index.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let patch_index_or_range_parts: Vec<&str> = s.split('-').collect();
        let num_parts = patch_index_or_range_parts.len();
        if num_parts == 1 {
            let patch_start_index_str = patch_index_or_range_parts.first().unwrap();
            let patch_start_index = patch_start_index_str.parse::<usize>().map_err(|e| {
                ParsePatchIndexOrRangeError::UnparsableIndex(patch_start_index_str.to_string(), e)
            })?;
            Ok(PatchIndexRange {
                start_index: patch_start_index,
                end_index: None,
            })
        } else if num_parts == 2 {
            let patch_start_index_str = patch_index_or_range_parts.first().unwrap();
            let patch_end_index_str = patch_index_or_range_parts.get(1).unwrap();
            let patch_start_index = patch_start_index_str.parse::<usize>().map_err(|e| {
                ParsePatchIndexOrRangeError::UnparsableIndex(patch_start_index_str.to_string(), e)
            })?;
            let patch_end_index = patch_end_index_str.parse::<usize>().map_err(|e| {
                ParsePatchIndexOrRangeError::UnparsableIndex(patch_end_index_str.to_string(), e)
            })?;
            if patch_end_index > patch_start_index {
                Ok(PatchIndexRange {
                    start_index: patch_start_index,
                    end_index: Some(patch_end_index),
                })
            } else {
                Err(ParsePatchIndexOrRangeError::StartPatchIndexLargerThanEnd(
                    s.to_string(),
                ))
            }
        } else {
            Err(ParsePatchIndexOrRangeError::InvalidIndexRange(
                s.to_string(),
            ))
        }
    }
}
