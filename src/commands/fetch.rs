use gps as ps;

pub fn fetch(color: bool) {
  match ps::fetch(color) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
