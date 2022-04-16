use gps as ps;

pub fn integrate(patch_index: usize, force: bool, keep_branch: bool, branch_name: Option<String>) {
  match ps::new_integrate::new_integrate(patch_index, force, keep_branch, branch_name) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
