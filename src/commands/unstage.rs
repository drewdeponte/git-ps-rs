use gps as ps;

pub fn unstage(files: Vec<std::string::String>) {
  match ps::unstage(files) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
