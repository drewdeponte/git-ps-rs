use super::utils::print_err;
use gps as ps;

pub fn integrate(
    patch_index: usize,
    force: bool,
    keep_branch: bool,
    branch_name: Option<String>,
    color: bool,
) {
    match ps::integrate::integrate(patch_index, force, keep_branch, branch_name, color) {
        Ok(_) => {}
        Err(ps::integrate::IntegrateError::PatchIsBehind) => {
            print_err(
                color,
                r#"
  The patch you are attempting to integrate is behind.

  This means that some change has been integrated into upstream since you last
  sync'd or requested review of this patch.

  To get caught up just sync or request-review the patch again.
"#,
            );
            std::process::exit(1);
        }
        Err(ps::integrate::IntegrateError::IsolationVerificationFailed(
            ps::VerifyIsolationError::IsolateFailed(ps::IsolateError::UncommittedChangesExist),
        )) => {
            print_err(
                color,
                r#"
  gps integrate command requires a clean working directory when verifying isolation, but it looks like yours is dirty.

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
