use super::super::super::ps;
use std::result::Result;

pub struct CherryPickRange {
    pub root_oid: git2::Oid,
    pub leaf_oid: Option<git2::Oid>,
}

#[derive(Debug)]
pub enum MapRangeForCherryPickError {
    StartIndexNotFound,
    EndIndexNotFound,
}

/// Convert from a `start_patch_index` and an optional `end_patch_index` into a `CherryPickRange`
/// containing a `root_oid` and optional `leaf_oid` that are ready to be used with the
/// cherry_pick() function.
///
/// Basically this maps either an individual patch index or patch series indexes from the world of
/// patch stack down to the world of Git so that the cherry_pick() operation can be performed.
///
/// The `start_patch_index` specifies either the only patch to include in the cherry pick or the
/// beginning patch of a patch series to be cherry picked. It is inclusive, meaning the patch
/// matching the index will be cherry picked.
///
/// The `end_patch_index` specifies the patch index to the patch that ends the series of patches
/// you want to cherry pick. It is inclusive, meaning the patch matching the index will be cherry
/// picked. If you want to only cherry pick the patch that matches the `start_patch_index` you can
/// simply set `end_patch_index` to `None`.
///
/// Returns a `CherryPickRange` containing a `root_oid` and `leaf_oid` ready to be used with the
/// cherry_pick() function to perform cherry pick either a single patch or a series of patches.
pub fn map_range_for_cherry_pick(
    patches_vec: &[ps::ListPatch],
    start_patch_index: usize,
    end_patch_index: Option<usize>,
) -> Result<CherryPickRange, MapRangeForCherryPickError> {
    let start_patch_oid = patches_vec
        .get(start_patch_index)
        .ok_or(MapRangeForCherryPickError::StartIndexNotFound)?
        .oid;

    Ok(match (start_patch_index, end_patch_index) {
        (si, Some(ei)) if (si < ei) => {
            let end_patch_oid = patches_vec
                .get(ei)
                .ok_or(MapRangeForCherryPickError::EndIndexNotFound)?
                .oid;
            CherryPickRange {
                root_oid: start_patch_oid,
                leaf_oid: Some(end_patch_oid),
            }
        }
        (si, Some(ei)) if (si > ei) => {
            let end_patch_oid = patches_vec
                .get(ei)
                .ok_or(MapRangeForCherryPickError::EndIndexNotFound)?
                .oid;
            CherryPickRange {
                root_oid: end_patch_oid,
                leaf_oid: Some(start_patch_oid),
            }
        }
        _ => CherryPickRange {
            root_oid: start_patch_oid,
            leaf_oid: None,
        },
    })
}
