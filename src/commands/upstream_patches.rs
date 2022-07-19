// This is the `upstream_patches` module. It is responsible for exposing a
// public interface making it easy for the CLI to execute the
// upstream-patches command. This generally means there is one public
// function. In this case the `upstream_patches()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules
// inside the ps module and should be exposed via the library public interface.

use gps as ps;

pub fn upstream_patches(color: bool) {
  match ps::upstream_patches(color) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
