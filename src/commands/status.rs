use gps as ps;

pub fn status() {
  match ps::status() {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
