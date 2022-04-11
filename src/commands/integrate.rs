use gps as ps;

pub fn integrate(patch_index: usize) {
  match ps::integrate::integrate(patch_index) {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
