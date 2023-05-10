#[cfg(feature = "backup_cmd")]
use gps as ps;
#[cfg(feature = "backup_cmd")]
use std::string::String;

#[cfg(feature = "backup_cmd")]
pub fn backup_stack(branch_name: String) {
    match ps::backup_stack(branch_name) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
