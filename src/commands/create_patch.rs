use gps as ps;

pub fn create_patch() {
  match ps::create_patch() {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
