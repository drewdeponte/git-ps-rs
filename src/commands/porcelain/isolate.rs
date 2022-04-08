use gps as ps;

pub fn isolate(patch_index: Option<usize>) {
  let res = ps::isolate(patch_index);
  match res {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
