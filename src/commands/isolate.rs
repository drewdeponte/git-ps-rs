use super::utils::print_err;
use gps as ps;

pub fn isolate(patch_index: Option<usize>, end_patch_index: Option<usize>, color: bool) {
    match ps::isolate(patch_index, end_patch_index, color) {
        Ok(_) => {}
        Err(e) => {
            print_err(color, format!("\nError: {:?}\n", e).as_str());
            std::process::exit(1);
        }
    }
}
