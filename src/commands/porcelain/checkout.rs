use gps as ps;

pub fn checkout(patch_index: Option<usize>) {
  let repo = ps::plumbing::git::create_cwd_repo().unwrap();
  let res = ps::checkout(&repo, patch_index);
  match res {
    Ok(_) => return,
    Err(e) => eprintln!("Error: {:?}", e)
  }
}
