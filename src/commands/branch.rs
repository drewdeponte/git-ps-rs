use gps as ps;
use std::option::Option;
use std::string::String;

pub fn branch(start_patch_index: usize, end_patch_index_option: Option<usize>, branch_name: Option<String>, create_remote: bool) {
  match ps::branch(start_patch_index, end_patch_index_option, branch_name, create_remote) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
