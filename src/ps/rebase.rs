use super::plumbing::utils;
use super::git;

pub fn rebase() {
  let repo = git::create_cwd_repo().unwrap();

  let head_ref = repo.head().unwrap();
  let head_branch_shorthand = head_ref.shorthand().unwrap();
  let head_branch_name = head_ref.name().unwrap();

  let upstream_branch_name = git::branch_upstream_name(&repo, head_branch_name).unwrap();

  match utils::execute("git", &["rebase", "-i", "--onto", upstream_branch_name.as_str(), upstream_branch_name.as_str(), head_branch_shorthand]) {
    Ok(_) => return,
    Err(e) => println!("error: {:?}", e)
  }
}
