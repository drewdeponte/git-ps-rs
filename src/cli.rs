use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct RequestReview {
  pub patch_index: usize,
  /// Use the provided branch name instead of generating one
  #[structopt(short = "n")]
  pub branch_name: Option<String>
}

#[derive(Debug, StructOpt)]
pub struct BranchCmdOpts {
  /// index of patch to cherry-pick to branch or starting index of patch
  /// series to cherry-pick to the branch
  pub start_patch_index: usize,
  /// ending patch index of the patch series to cherry-pick to the branch
  pub end_patch_index: Option<usize>,
  /// Use the provided branch name instead of generating one
  #[structopt(short = "n")]
  pub branch_name: String
}

#[derive(Debug, StructOpt)]
pub struct RequestReviewBranchCmdOpts {
  pub patch_index: usize,
  /// Use the provided branch name instead of generating one
  #[structopt(short = "n")]
  pub branch_name: Option<String>
}

#[derive(Debug, StructOpt)]
pub struct IntegrateCmdOpts {
  pub patch_index: usize,
  /// Use the provided branch name instead of generating one
  #[structopt(short = "n")]
  pub branch_name: Option<String>,
  /// Keep request-review branches around (a.k.a. don't clean up request
  /// review branches)
  #[structopt(short = "k", long = "keep-branch")]
  pub keep_branch: bool,
  /// Skip safety checks and publish
  #[structopt(short = "f", long = "force")]
  pub force: bool
}

#[derive(Debug, StructOpt)]
pub struct ShowCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub struct SyncCmdOpts {
  pub patch_index: usize,
  /// Use the provided branch name instead of generating one
  #[structopt(short = "n")]
  pub branch_name: Option<String>
}

#[derive(Debug, StructOpt)]
pub struct IsolateCmdOpts {
  pub patch_index: Option<usize>
}

#[derive(Debug, StructOpt)]
pub struct CheckoutCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Your bridge back to the world of normal git and git concepts.
    /// Basically a utility to help you create a normal git branch from a
    /// patch or series of patches that is based on the patch stack base (e.g.
    /// origin/main). Because this is a bridge back to the normal git concepts
    /// like branches and commits it does no state tracking of these branches
    /// inside of git patch stack.
    Branch(BranchCmdOpts),
    /// Create a request review branch on the patch stack base, cherry-pick
    /// the specified patch onto it, & record association between patch &
    /// branch
    #[structopt(name = "request-review-branch")]
    RequestReviewBranch(RequestReviewBranchCmdOpts),
    /// (int) - Integrate the specified patch into the patch stacks upstream
    /// remote
    #[structopt(name = "integrate", alias = "int")]
    Integrate(IntegrateCmdOpts),
    /// (ls) - List the stack of patches and their associated state info
    #[structopt(name = "list", alias = "ls")]
    List,
    /// Interactively rebase your stack of patches
    #[structopt(name = "rebase")]
    Rebase,
    /// Pull changes down from upstream and rebase stack on top
    #[structopt(name = "pull")]
    Pull,
    /// (rr) - Request review of the specified patch
    #[structopt(name = "request-review", alias = "rr")]
    RequestReview(RequestReview),
    /// Show the identified patch in raw form
    #[structopt(name = "show")]
    Show(ShowCmdOpts),
    /// Synchronize patch with the remote
    #[structopt(name = "sync")]
    Sync(SyncCmdOpts),
    /// (iso) - Isolate a patch by creating a temporary branch based on
    /// upstream, cherry-picking the patch to it, & checking it out
    #[structopt(name = "isolate", alias = "iso")]
    Isolate(IsolateCmdOpts),
    /// (co) - Checkout the patch identified by the patch-index, leaving you
    /// in a headless state.
    #[structopt(name = "checkout", alias = "co")]
    Checkout(CheckoutCmdOpts),
    /// (c) - create a patch from the currently staged changes
    #[structopt(name = "create-patch", alias = "c")]
    CreatePatch
}

#[derive(Debug, StructOpt)]
#[structopt(name = "gps")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command
}
