use super::patch_index_range::PatchIndexRange;
use super::utils::{print_err, print_error_chain};
use gps as ps;
use std::str::FromStr;

pub fn integrate(
    patch_index_or_range: String,
    force: bool,
    keep_branch: bool,
    branch_name: Option<String>,
    color: bool,
) {
    match PatchIndexRange::from_str(&patch_index_or_range) {
        Ok(patch_index_range) => {
            match ps::integrate::integrate(
                patch_index_range.start_index,
                patch_index_range.end_index,
                force,
                keep_branch,
                branch_name,
                color,
            ) {
                Ok(_) => {}
                Err(ps::integrate::IntegrateError::MergeCommitDetected(oid)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Detected a merge commit ({}) in the patch(es) selection.

  This should only occur if you have a merge commit that hasn't been flatten in your patch stack.
  To flatten merge commits in your patch stack you can simply run gps rebase and then try to integrate again.
        "#,
                            oid
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::ConflictsExist(src_oid, dst_oid)) => {
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
                Err(ps::integrate::IntegrateError::HasNoAssociatedBranch) => {
                    print_err(
                        color,
                        r#"
  Looks like the patch(es) haven't had a branch associated with them yet.

  You can associate them with gps request-review-branch and gps sync.

  Or you can skip saftey checks like this one with --force.
        "#,
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::AssociatedBranchAmbiguous) => {
                    print_err(
                        color,
                        r#"
  Looks like the patch(es) are associated to more than one branch.

  To integrate it needs to know which associated branch you want to work with. You can specify the branch with -n.

  Alternatively you can also skip saftey checks like this one with --force.
        "#,
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::UpstreamBranchInfoMissing) => {
                    print_err(
                        color,
                        r#"
  Looks like the patch(es) have not been pushed yet.

  Either push them with gps sync or gps request-review.

  Alternatively you can also skip saftey checks like this one with --force.
        "#,
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::CommitCountMissmatch(
                    patch_series_count,
                    remote_branch_commit_count,
                )) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  Looks like the patch series consists of {} patches but the remote has {}.

  Investigate this mismatch & resolve it before trying to integrate again.

  Or you can skip saftey checks like this one with --force.
        "#,
                            patch_series_count, remote_branch_commit_count
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::PatchAndRemotePatchIdMissmatch(patch_index)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  The patch at index {} has a different patch identifier than the patch on the remote.

  It could be that you made a change locally to that patches id or that someone made a change to that patches id remotely.

  If you made the change locally, you can update the remote with gps sync or gps request-review.

  Alternatively you can also skip saftey checks like this one with --force.
        "#,
                            patch_index
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::PatchDiffHashMissmatch(patch_index)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  The patch at index {} is different from that patch on the remote.

  It could be that you made a change locally to that patch or that someone made a change to that patch remotely.

  If you made a change locally, you can update the remote with gps sync or gps request-review.

  Alternatively you can also skip saftey checks like this one with --force.
        "#,
                            patch_index
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::VerifyHookExecutionFailed(e)) => {
                    print_err(
                        color,
                        &format!(
                            r#"
  The integrate_verify hook execution failed with {:?}.

  We have aborted the integration. For the integration to continue the integrate_verify hook must exit 0.

  Alternatively you can also skip saftey checks like this one with --force.
        "#,
                            e
                        ),
                    );
                    std::process::exit(1);
                }
                Err(ps::integrate::IntegrateError::UncommittedChangesExist) => {
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
