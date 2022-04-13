use gps as ps;

pub fn integrate(patch_index: usize, keep_branch: bool, branch_name: Option<String>) {
  // match ps::new_integrate::new_integrate(patch_index, keep_branch, branch_name) {
  //   Ok(_) => {},
  //   Err(e) => eprintln!("Error: {:?}", e)
  // }
  match ps::integrate::integrate(patch_index, keep_branch, branch_name) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
