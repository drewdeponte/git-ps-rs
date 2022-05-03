use gps as ps;

pub fn pull(color: bool) {
  match ps::pull(color) {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
