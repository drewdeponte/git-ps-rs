// This is the `rebase` module. It is responsible for exposing a public
// interface making it easy for the CLI to execute the rebase command. This
// generally means there is one public function. In this case the `rebase()`
// function. All other functions in here are purely private supporting
// functions and should be strongly considered if they fit better in one of
// the other modules in the `ps` module.

use gps as ps;

pub fn rebase(continue_rebase: bool) {
    match ps::rebase(continue_rebase) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    };
}
