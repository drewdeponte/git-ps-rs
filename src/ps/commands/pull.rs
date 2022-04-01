// This is the `pull` module. It is responsible for exposing a public
// interface making it easy for the CLI to execute the ls subcommand. This
// generally means there is one public function. In this case the `pull()`
// function. All other functions in here are purely private supporting
// functions and should be strongly considered if they fit better in one of
// the other modules such as the `ps::ps`, `ps::git`, or `ps::utils`.

use super::super::utils;

pub fn pull() {
  match utils::execute("git", &["pull", "--rebase"]) {
    Ok(_) => return,
    Err(e) => println!("error: {:?}", e)
  }
}
