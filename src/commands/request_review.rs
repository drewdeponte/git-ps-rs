// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr command. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules
// inside of the `ps` module.

use gps as ps;
use ansi_term::Colour::Red;

pub fn request_review(patch_index: usize, branch_name: Option<String>, color: bool) {
  match ps::request_review(patch_index, branch_name) {
    Ok(_) => {},
    Err(e) => {
      let error_string = format!("\n\n{}\n\n", e);
      if color {
        eprintln!("{}", Red.paint(error_string))
      } else {
        eprintln!("{}", error_string)
      }
    }
  };
}
