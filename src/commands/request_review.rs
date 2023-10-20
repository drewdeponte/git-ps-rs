// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr command. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules
// inside of the `ps` module.

use super::patch_index_range::PatchIndexRange;
use super::utils::print_err;
use gps as ps;
use std::str::FromStr;

pub fn request_review(
    patch_index_or_range: String,
    branch_name: Option<String>,
    color: bool,
    isolation_verification_hook: bool,
    post_sync_hook: bool,
) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            match ps::request_review(
                patch_index_range.start_index,
                patch_index_range.end_index,
                branch_name,
                color,
                isolation_verification_hook,
                post_sync_hook,
            ) {
                Ok(_) => {}
                Err(ps::RequestReviewError::PostSyncHookNotFound) => print_err(
                    color,
                    r#"
  The request_review_post_sync hook was not found!

  This hook is required to be installed and configured for the
  request-review command. It is executed after the patch is successfully
  sync'd to the remote.

  It is responsible for requesting review of the patch within your
  preferred code review platform & workflow. For example if you worked
  on the Git or Linux Kernel dev teams you might want it to format your
  patch as an email and send it to the appropriate mailing list. If on
  the other hand you use GitHub and pull requests for code reviews you
  could have it simply create the pull request for you on GitHub.

  You can effectively have it do whatever you want as it is just a hook.
  An exit status of 0, success, informs gps that it should update it's
  state tracking for that patch to indicate that it has been requested
  for review. Any non-zero exit status will indicate failure and cause
  gps to abort and not update the patch's state.

  You can find more information and examples of this hook and others at
  the following.

  https://github.com/uptech/git-ps-rs#hooks
"#,
                ),
                Err(ps::RequestReviewError::PostSyncHookNotExecutable(path)) => {
                    let path_str = path.to_str().unwrap_or("unknow path");
                    let msg = format!(
                        r#"
  The request_review_post_sync hook was found at

    {}

  but it is NOT executable. For the request-review command to function properly
  the hook needs to be executable. Generally this can be done with the
  following.

    chmod u+x {}
"#,
                        path_str, path_str
                    );
                    print_err(color, &msg);
                    std::process::exit(1);
                }
                Err(ps::RequestReviewError::IsolationVerificationFailed(
                    ps::VerifyIsolationError::IsolateFailed(
                        ps::IsolateError::UncommittedChangesExist,
                    ),
                )) => {
                    print_err(
                        color,
                        r#"
  gps request-review command requires a clean working directory when verifying isolation, but it looks like yours is dirty.

  It is recommended that you create a WIP commit. But, you could also use git stash if you prefer.
        "#,
                    );
                    std::process::exit(1);
                }
                Err(e) => {
                    print_err(color, format!("\nError: {}\n", e).as_str());
                    std::process::exit(1);
                }
            };
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
