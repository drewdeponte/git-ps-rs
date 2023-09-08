use super::utils::print_err;
use gps as ps;

pub fn isolate(patch_index: Option<usize>, end_patch_index: Option<usize>, color: bool) {
    match ps::isolate(patch_index, end_patch_index, color) {
        Ok(_) => {}
        Err(ps::IsolateError::UncommittedChangesExist) => {
            print_err(
                color,
                r#"
  gps isolate requires a clean working directory but it looks like yours is dirty.

  It is recommended that you create a WIP commit. But, you could also use git stash if you prefer.
        "#,
            );
            std::process::exit(1);
        }
        Err(e) => {
            print_err(color, format!("\nError: {:?}\n", e).as_str());
            std::process::exit(1);
        }
    }
}
