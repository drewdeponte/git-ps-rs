// This is the `rebase` module. It is responsible for exposing a public
// interface making it easy for the CLI to execute the rebase subcommand. This
// generally means there is one public function. In this case the `rebase()`
// function. All other functions in here are purely private supporting
// functions and should be strongly considered if they fit better in one of
// the other modules such as the `ps::ps`, `ps::git`, or `ps::utils`.

use super::super::utils;
use git2;

pub fn rebase() {
  let repo = match git2::Repository::discover("./") {
      Ok(repo) => repo,
      Err(e) => panic!("failed to open: {}", e),
  };

  let head_ref = repo.head().unwrap();
  let head_branch_shorthand = head_ref.shorthand().unwrap();

  let head_branch_name = head_ref.name().unwrap();
  let upstream_branch_name_buf = repo.branch_upstream_name(head_branch_name).unwrap();
  let upstream_branch_name = upstream_branch_name_buf.as_str().unwrap();

  let res = utils::execute("git", &["rebase", "-i", "--onto", upstream_branch_name, upstream_branch_name, head_branch_shorthand]);
  match res {
    Ok(exit_status) => println!("exitStatus: {}", exit_status),
    Err(e) => println!("error: {}", e)
  }
}
