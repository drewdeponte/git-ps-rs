// This is the `rr` module. It is responsible for exposing a public interface
// making it easy for the CLI to execute the rr command. This generally
// means there is one public function. In this case the `rr()` function. All
// other functions in here are purely private supporting functions and should
// be strongly considered if they fit better in one of the other modules
// inside of the `ps` module.

use super::patch_index_range_batch::PatchIndexRangeBatch;
use super::utils::{print_err, print_error_chain};
use gps as ps;

pub fn request_review(
    patch_index_or_range_batch: String,
    branch_name: Option<String>,
    color: bool,
    isolation_verification_hook: bool,
    post_sync_hook: bool,
) {
    let batch = match patch_index_or_range_batch.parse::<PatchIndexRangeBatch>() {
        Ok(b) => b,
        Err(e) => {
            print_err(
                color,
                &format!(
                    "Failed to parse patch index range batch, \"{}\"",
                    &patch_index_or_range_batch
                ),
            );
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    };

    if batch.len() > 1 && branch_name.is_some() {
        print_err(
            color,
            r#"
  Specifying a branch name does not work when batching.

  You can associate custom branch names with patches or patch series by
  using the branch command prior to requesting review of them as a batch.
  The request-review command will use the associated branch name for each
  patch or patch series respectively.

  For patches or patch series that you have not previously associated a
  custom branch name to. The request-review command will simply generate
  a branch name for each of them.
"#,
        );
        std::process::exit(1);
    }

    for patch_index_range in batch {
        println!("Running request-review for {}", patch_index_range);
        match ps::request_review(
            patch_index_range.start_index,
            patch_index_range.end_index,
            branch_name.clone(),
            color,
            isolation_verification_hook,
            post_sync_hook,
        ) {
            Ok(_) => {}
            Err(e) => match e {
                ps::RequestReviewError::MergeCommitDetected(oid) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Detected a merge commit ({}) in the patch(es) selection.

  This should only occur if you have a merge commit that hasn't been flatten in your patch stack.
  To flatten merge commits in your patch stack you can simply run gps rebase and then try to request review again.
        "#,
                            oid
                        ),
                    );
                }
                ps::RequestReviewError::ConflictsExist(src_oid, dst_oid) => {
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
                }
                ps::RequestReviewError::PostSyncHookNotExecutable(path) => {
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
                }
                ps::RequestReviewError::IsolationVerificationFailed(
                    ps::VerifyIsolationError::IsolateFailed(
                        ps::IsolateError::UncommittedChangesExist,
                    ),
                ) => {
                    print_err(
                        color,
                        r#"
  gps request-review command requires a clean working directory when verifying isolation, but it looks like yours is dirty.

  It is recommended that you create a WIP commit. But, you could also use git stash if you prefer.
        "#,
                    );
                }
                _ => {
                    print_error_chain(color, e.into());
                }
            },
        };
    }
}
