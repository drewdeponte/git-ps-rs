use gps as ps;

pub fn checkout(patch_index: usize) {
  let res = ps::checkout(patch_index);
  match res {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  }
}
