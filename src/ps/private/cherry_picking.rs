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

pub fn map_range_for_cherry_pick(
    patches_vec: &Vec<ps::ListPatch>,
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
