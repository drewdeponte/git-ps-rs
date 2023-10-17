use super::patch_index_range::PatchIndexRange;
use gps as ps;
use std::option::Option;
use std::str::FromStr;
use std::string::String;

pub fn request_review_branch(patch_index_or_range: String, branch_name: Option<String>) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            let res = ps::request_review_branch(
                patch_index_range.start_index,
                patch_index_range.end_index,
                branch_name,
            );
            match res {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
