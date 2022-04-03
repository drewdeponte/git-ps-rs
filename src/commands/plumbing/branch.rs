use gps as ps;

pub fn branch(patch_index: usize) {
  let repo = ps::plumbing::git::create_cwd_repo().unwrap();
  let res = ps::plumbing::branch::branch(&repo, patch_index);
  match res {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {}", e)
  }
}
