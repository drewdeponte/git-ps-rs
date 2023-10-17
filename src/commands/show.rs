use super::patch_index_range::PatchIndexRange;
use gps as ps;
use std::str::FromStr;

pub fn show(patch_index_or_range: String) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            match ps::show(patch_index_range.start_index, patch_index_range.end_index) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            };
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
