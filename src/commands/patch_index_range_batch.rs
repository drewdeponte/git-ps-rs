use std::collections::VecDeque;

use super::patch_index_range::{ParsePatchIndexOrRangeError, PatchIndexRange};

#[derive(Debug, Clone)]
pub struct PatchIndexRangeBatch {
    patch_index_ranges: VecDeque<PatchIndexRange>,
}

#[derive(Debug)]
pub enum ParsePatchIndexRangeBatchError {
    ParsePatchIndexOrRangeFailed(ParsePatchIndexOrRangeError),
}

impl std::fmt::Display for ParsePatchIndexRangeBatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsePatchIndexOrRangeFailed(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ParsePatchIndexRangeBatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParsePatchIndexOrRangeFailed(e) => Some(e),
        }
    }
}

impl PatchIndexRangeBatch {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.patch_index_ranges.len()
    }
}

impl std::str::FromStr for PatchIndexRangeBatch {
    type Err = ParsePatchIndexRangeBatchError;

    /// Parse string representation of an a batch of patch indexes & ranges
    ///
    /// A patch index style would simply be a string that is the patch index, e.g. "12". A patch index
    /// range on the other hand is two patch index strings separated by a dash character, e.g. "2-4".
    /// The left most patch index is the starting patch index and the right most patch index is the
    /// ending patch index. A batch is a collection of patch indexes and patch index rages
    /// separated by spaces, e.g. "1 2-4 8". In this example it would result in three
    /// PatchIndexRange objects, one representing the 1, another representing the 2 to 4 range, and
    /// another representing the 8. This interprets this string representation and initializes a
    /// PatchIndexRangeBatch object from it.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut index_ranges: Vec<PatchIndexRange> = vec![];

        for index_or_range_str in s.split(' ') {
            let index_range = index_or_range_str
                .parse::<PatchIndexRange>()
                .map_err(ParsePatchIndexRangeBatchError::ParsePatchIndexOrRangeFailed)?;
            index_ranges.push(index_range);
        }

        Ok(PatchIndexRangeBatch {
            patch_index_ranges: VecDeque::from(index_ranges),
        })
    }
}

impl std::iter::Iterator for PatchIndexRangeBatch {
    type Item = PatchIndexRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.patch_index_ranges.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::{
        patch_index_range::PatchIndexRange, patch_index_range_batch::PatchIndexRangeBatch,
    };

    #[test]
    fn smoke_test_parse_patch_index_range_batch() {
        let mut batch: PatchIndexRangeBatch =
            "0 2-4 5 8-9".parse::<PatchIndexRangeBatch>().unwrap();
        assert_eq!(batch.len(), 4);

        let first = batch.next().unwrap();
        assert_eq!(
            first,
            PatchIndexRange {
                start_index: 0,
                end_index: None,
            },
        );

        let second = batch.next().unwrap();
        assert_eq!(
            second,
            PatchIndexRange {
                start_index: 2,
                end_index: Some(4),
            },
        );

        let third = batch.next().unwrap();
        assert_eq!(
            third,
            PatchIndexRange {
                start_index: 5,
                end_index: None,
            },
        );

        let fourth = batch.next().unwrap();
        assert_eq!(
            fourth,
            PatchIndexRange {
                start_index: 8,
                end_index: Some(9),
            },
        );

        let fifth = batch.next();
        assert!(fifth.is_none());
    }
}
