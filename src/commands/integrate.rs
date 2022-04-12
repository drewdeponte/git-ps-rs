use gps as ps;

pub fn integrate(patch_index: usize, branch_name: Option<String>) {
  match ps::integrate::integrate(patch_index, branch_name) {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
