use gps as ps;
use std::string::String;

#[allow(dead_code)]
pub fn backup_stack(branch_name: String) {
  match ps::backup_stack(branch_name) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
