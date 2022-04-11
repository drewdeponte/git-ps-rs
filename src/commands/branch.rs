use gps as ps;

pub fn branch(patch_index: usize) {
  let res = ps::branch(patch_index);
  match res {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
