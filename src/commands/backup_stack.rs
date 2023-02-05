use gps as ps;
use std::string::String;

pub fn backup_stack(branch_name: String) {
  match ps::backup_stack(branch_name) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
