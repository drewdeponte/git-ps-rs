use gps as ps;
use std::option::Option;
use std::string::String;

pub fn request_review_branch(patch_index: usize, branch_name: Option<String>) {
    let res = ps::request_review_branch(patch_index, branch_name);
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
