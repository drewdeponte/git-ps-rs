use super::patch_index_range::PatchIndexRange;
use super::utils::print_err;
use gps as ps;
use std::str::FromStr;

pub fn isolate(patch_index_or_range: Option<String>, color: bool) {
    match patch_index_or_range {
        Some(pir) => match PatchIndexRange::from_str(&pir) {
            Ok(patch_index_range) => match ps::isolate(
                Some(patch_index_range.start_index),
                patch_index_range.end_index,
                color,
            ) {
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
            },
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        },
        None => match ps::isolate(None, None, color) {
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
        },
    }
}
