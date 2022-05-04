use gps as ps;

pub fn integrate(patch_index: usize, force: bool, keep_branch: bool, branch_name: Option<String>, color: bool) {
  match ps::integrate::integrate(patch_index, force, keep_branch, branch_name, color) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
