use std::option::Option;
use std::string::String;

#[derive(Debug, Clone)]
pub struct PatchIndexRange {
    pub start_index: usize,
    pub end_index: Option<usize>,
}

impl PartialEq for PatchIndexRange {
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index && self.end_index == other.end_index
    }
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

// Tests

// Parsing of single patch indexes

#[test]
fn single1() {
    let patch_index_range = "1".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_range,
        PatchIndexRange {
            start_index: 1,
            end_index: None,
        },
    );
}

#[test]
fn single2() {
    let patch_index_or_range = "12".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: 12,
            end_index: None,
        },
    );
}

#[test]
fn single3() {
    let patch_index_or_range = "12341234123412341234".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: 12341234123412341234,
            end_index: None,
        },
    );
}

#[test]
fn single4() {
    let patch_index_or_range = "0".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: 0,
            end_index: None,
        },
    );
}

#[test]
fn single5() {
    let patch_index_or_range = "-1".parse::<PatchIndexRange>();
    assert!(patch_index_or_range.is_err());
}

#[test]
fn single6() {
    let patch_index_or_range = u32::MAX.to_string().parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: u32::MAX as usize,
            end_index: None,
        },
    );
}

// Parsing of ranges

#[test]
fn range1() {
    let patch_index_or_range = "2-4".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: 2,
            end_index: Some(4),
        },
    );
}

#[test]
fn range2() {
    let patch_index_or_range = "2-333".parse::<PatchIndexRange>().unwrap();
    assert_eq!(
        patch_index_or_range,
        PatchIndexRange {
            start_index: 2,
            end_index: Some(333),
        },
    );
}

// CR-someday alizter: We don't accept reflexive ranges, maybe we should?
// It seems simple enought to parse this as a single patch index.
#[test]
fn range3() {
    let patch_index_or_range = "2-2".parse::<PatchIndexRange>();
    assert!(patch_index_or_range.is_err());
}

#[test]
fn range4() {
    let patch_index_or_range = "4-2".parse::<PatchIndexRange>();
    assert!(patch_index_or_range.is_err());
}

#[test]
fn range5() {
    let patch_index_range = "0--1".parse::<PatchIndexRange>();
    assert!(patch_index_range.is_err());
}

#[test]
fn range6() {
    let patch_index_range = "-1-2".parse::<PatchIndexRange>();
    assert!(patch_index_range.is_err());
}

// Invalid syntax

#[test]
fn malformed1() {
    let patch_index_or_range = "2-4-6".parse::<PatchIndexRange>();
    assert!(patch_index_or_range.is_err());
}

#[test]
fn malformed2() {
    let patch_index_range = "2-".parse::<PatchIndexRange>();
    assert!(patch_index_range.is_err());
}

#[test]
fn malformed3() {
    let patch_index_range = "".parse::<PatchIndexRange>();
    assert!(patch_index_range.is_err());
}

#[test]
fn malformed4() {
    let patch_index_range = "-0".parse::<PatchIndexRange>();
    assert!(patch_index_range.is_err());
}
