use gps as ps;
use std::option::Option;
use std::string::String;

pub fn branch(patch_index: usize, branch_name: Option<String>) {
  let res = ps::branch(patch_index, branch_name);
  match res {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
