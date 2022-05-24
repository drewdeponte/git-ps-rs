// This is the `ls` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the ls command. This generally
// means there is one public function. In this case the `ls()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules
// inside the ps module and should be exposed via the library public interface.

use gps as ps;
use std::thread;

pub fn list(color: bool) {
  let check_release_thread = thread::spawn(ps::newer_release_available);

  match ps::list(color) {
    Ok(_) => {},
    Err(e) => eprintln!("Error: {:?}", e)
  };

  match check_release_thread.join().expect("Check release thread panicked!") {
    Ok(newer_release) => ps::notify_of_newer_release(newer_release, color),
    Err(e) => eprintln!("Error: {:?}", e)
  };
}
