use gps as ps;
use std::option::Option;
use std::string::String;

use super::utils::print_err;

pub fn branch(
    start_patch_index: usize,
    end_patch_index_option: Option<usize>,
    branch_name: Option<String>,
    push_to_remote: bool,
    color: bool,
) {
    match ps::branch(
        start_patch_index,
        end_patch_index_option,
        branch_name,
        push_to_remote,
        color,
    ) {
        Ok(_) => {}
        Err(ps::BranchError::IsolationVerificationFailed(
            ps::VerifyIsolationError::IsolateFailed(ps::IsolateError::UncommittedChangesExist),
        )) => {
            print_err(
                color,
                r#"
  gps branch command requires a clean working directory when verifying isolation. It looks like yours is dirty.

  It is recommended that you create a WIP commit. But, you could also use git stash if you prefer.
        "#,
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
