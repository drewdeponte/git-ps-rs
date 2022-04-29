use gps as ps;

pub fn amend_patch() {
  match ps::amend_patch() {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
