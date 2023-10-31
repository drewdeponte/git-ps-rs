use super::patch_index_range::PatchIndexRange;
use super::utils::{print_err, print_error_chain};
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
                Err(ps::IsolateError::MergeCommitDetected(oid)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Detected a merge commit ({}) in the patch(es) to isolate.

  This should only occur if you have a merge commit that hasn't been flatten in your patch stack.
  To flatten merge commits in your patch stack you can simply run gps rebase and then try to isolate again.
        "#,
                            oid
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::IsolateError::ConflictsExist(src_oid, dst_oid)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Cherry picking commit ({}) onto commit ({}) failed due to conflicts.

  Please make sure that you aren't missing a dependent patch in your patch(es) selection.
        "#,
                            src_oid, dst_oid
                        ),
                    );
                    std::process::exit(1);
                }
                Err(e) => {
                    print_error_chain(color, e.into());
                    std::process::exit(1);
                }
            },
            Err(e) => {
                print_error_chain(color, e.into());
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
                print_error_chain(color, e.into());
                std::process::exit(1);
            }
        },
    }
}
