use gps as ps;
use std::thread;

pub fn pull(color: bool) {
  let check_release_thread = thread::spawn(ps::newer_release_available);

  match ps::pull(color) {
    Ok(_) => {},
    Err(e) => {
      eprintln!("Error: {:?}", e);
      std::process::exit(1);
    }
  };

  if let Ok(newer_release) = check_release_thread.join().expect("Check release thread panicked!") {
    ps::notify_of_newer_release(newer_release, color);
  }
}
