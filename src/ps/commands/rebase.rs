// This is the `rebase` module. It is responsible for exposing a public
// interface making it easy for the CLI to execute the rebase subcommand. This
// generally means there is one public function. In this case the `rebase()`
// function. All other functions in here are purely private supporting
// functions and should be strongly considered if they fit better in one of
// the other modules such as the `ps::ps`, `ps::git`, or `ps::utils`.

use super::super::utils;
use super::super::git;

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
