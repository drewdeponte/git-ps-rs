use gps as ps;

pub fn branch(patch_index: usize) {
  match ps::branch(patch_index) {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {}", e)
  }
}
