use super::patch_index_range::PatchIndexRange;
use super::utils::{print_err, print_error_chain};
use gps as ps;
use std::option::Option;
use std::str::FromStr;
use std::string::String;

pub fn branch(patch_index_or_range: String, branch_name: Option<String>, color: bool) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            let res = ps::branch(
                patch_index_range.start_index,
                patch_index_range.end_index,
                branch_name,
            );
            match res {
                Ok(_) => {}
                Err(ps::BranchError::MergeCommitDetected(oid)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Detected a merge commit ({}) in the patch(es) selection.

  This should only occur if you have a merge commit that hasn't been flatten in your patch stack.
  To flatten merge commits in your patch stack you can simply run gps rebase and then try to create the branch again.
        "#,
                            oid
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::BranchError::ConflictsExist(src_oid, dst_oid)) => {
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
            }
        }
        Err(e) => {
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    }
}
