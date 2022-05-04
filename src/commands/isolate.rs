use gps as ps;
use super::utils::print_err;

pub fn isolate(patch_index: Option<usize>, color: bool) {
  match ps::isolate(patch_index, color) {
    Ok(_) => {},
    Err(e) => print_err(color, format!("\nError: {:?}\n", e).as_str())
  }
}
