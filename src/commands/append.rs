use super::patch_index_range::PatchIndexRange;
use super::utils::print_error_chain;
use gps as ps;
use std::str::FromStr;

pub fn append(patch_index_or_range: String, branch_name: String, color: bool) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            let res = ps::append::append(
                patch_index_range.start_index,
                patch_index_range.end_index,
                branch_name,
            );
            match res {
                Ok(_) => {}
                Err(e) => {
                    print_error_chain(color, e.into());
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    }
}
