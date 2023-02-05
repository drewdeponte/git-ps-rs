use gps as ps;
use super::utils::print_err;

pub fn integrate(patch_index: usize, force: bool, keep_branch: bool, branch_name: Option<String>, color: bool) {
  match ps::integrate::integrate(patch_index, force, keep_branch, branch_name, color) {
    Ok(_) => {},
    Err(ps::integrate::IntegrateError::PatchIsBehind) => {
      print_err(color,
r#"
  The patch you are attempting to integrate is behind.

  This means that some change has been integrated into upstream since you last
  sync'd or requested review of this patch.

  To get caught up just sync or request-review the patch again.
"#);
      std::process::exit(1);
    }
    Err(e) => {
      print_err(color, format!("\nError: {:?}\n", e).as_str());
      std::process::exit(1);
    }
  }
}
