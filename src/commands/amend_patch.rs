use gps as ps;

pub fn amend_patch(no_edit: bool) {
  match ps::amend_patch(no_edit) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
