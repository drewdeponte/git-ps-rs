use gps as ps;

pub fn pull() {
  match ps::pull() {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
