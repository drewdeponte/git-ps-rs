use gps as ps;

pub fn sync(patch_index: usize) {
  let res = ps::sync(patch_index);
  match res {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
