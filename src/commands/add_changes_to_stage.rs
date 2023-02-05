use gps as ps;

pub fn add_changes_to_stage(interactive: bool, patch: bool, edit: bool, all: bool, files: Vec<std::string::String>) {
  match ps::add_changes_to_stage(interactive, patch, edit, all, files) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
