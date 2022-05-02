use gps as ps;

pub fn log() {
  match ps::log() {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
