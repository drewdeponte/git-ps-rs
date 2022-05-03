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
pub struct AddCmdOpts {
  /// interactive picking
  #[structopt(short = "i", long = "interactive")]
  pub interactive: bool,
  /// select hunks interactively
  #[structopt(short = "p", long = "patch")]
  pub patch: bool,
  /// edit current diff and apply
  #[structopt(short = "e", long = "edit")]
  pub edit: bool,
  /// add changes from all tracked and untracked files
  #[structopt(short = "A", long = "all")]
  pub all: bool,
  /// specific files to add changes from, . for all files
  pub files: Vec<String>
}

#[derive(Debug, StructOpt)]
pub struct RebaseCmdOpts {
  /// continue a rebase that was paused
  #[structopt(long = "continue")]
  pub r#continue: bool
}

#[derive(Debug, StructOpt)]
pub struct UnstageCmdOpts {
  /// specific files to unstage changes for, leave blank for all staged files
  pub files: Vec<String>
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
    ///
    /// The `rebase` command initiates an interactive rebase to allow you to
    /// modify your stack of patches. This could be simply re-ordering them or
    /// modifying a particular patch in the stack. Or doing a plethora of
    /// other things that interactive rebases support.
    ///
    /// Some of those operations drop you out to the working copy in a rebase
    /// paused state so that you can make changes. This happens for example
    /// with the `edit` command in the interactive rebase.
    ///
    /// To resume the rebase after making your necessary changes you can do so
    /// by running `gps rebase --continue`.
    #[structopt(name = "rebase")]
    Rebase(RebaseCmdOpts),

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

    /// (iso) - isolate a patch for manual testing or evaluation.
    ///
    /// The `isolate` command isolates a patch for manual testing or
    /// evaluation by making sure no uncommitted changes exist, creating a
    /// temporary branch based on the patch stacks base, cherry-picking the
    /// patch to it, and then checking out that branch.
    ///
    /// If you have the `isolate_post_checkout` hook setup then that will be
    /// executed after successfully checking out the branch.
    ///
    /// When you are done manually testing or evaluating the patch in
    /// isolation you can return to the stack that you were on when you
    /// switch into isolation mode by running `gps iso`, basically leaving the
    /// index off.
    #[structopt(name = "isolate", alias = "iso")]
    Isolate(IsolateCmdOpts),
    /// (co) - Checkout the patch identified by the patch-index, leaving you
    /// in a headless state.
    #[structopt(name = "checkout", alias = "co")]
    Checkout(CheckoutCmdOpts),
    /// (c) - create a patch from the currently staged changes
    #[structopt(name = "create-patch", alias = "c")]
    CreatePatch,
    /// (a) - amend the top most patch with the currently staged changes
    #[structopt(name = "amend-patch", alias = "a")]
    AmendPatch,

    /// (s) - get the status of local changes & staged changes
    #[structopt(name = "status", alias = "s")]
    Status,

    /// add changes to the stage (a.k.a stage local changes)
    Add(AddCmdOpts),

    /// display a log of integrated patches
    Log,

    /// unstage currently staged changes
    Unstage(UnstageCmdOpts)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "gps")]
pub struct ApplicationArguments {
  /// disable color output
  #[structopt(long = "no-color", parse(from_flag = std::ops::Not::not))]
  pub color: bool,

  #[structopt(subcommand)]
  pub command: Command
}
