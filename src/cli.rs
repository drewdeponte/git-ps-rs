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
    #[structopt(name = "list", alias = "ls", long_about=r"
(ls) - List the stack of patches and their associated state info

The `list` command lists out your stack of patches in a format that exposes
the patch index on the far left followed by the state information, followed by
the short sha of the git commit, and finally followed by the patch summary.

[index] [status]    [sha] [summary]

The patch index value is used with other commands, e.g. `gps rr
<patch-index>`.

The state information is broken down into main states and modifiers. The main
states are as follows.

b    - local branch has been created with the patch
s    - the patch has been synced to the remote
rr   - you have requested review of the patch
int  - you have integrated the patch into upstream

Each of the states can have any of the following modifiers.

+    - the patch in your patch stack has changed since the operation
!    - the remote patch has changed since the operation
↓    - patch is behind, the patch stack base has been updated since the operation

To fully understand this lets look at an example. Let say you see the status
`rr+!` when you ran the `list` command. This is telling you that the current
patch is in a state of **requested review**, indicated by the `rr`. It is also
telling you that since you last requested review of that patch changes have
been made to it, in your patch stack. This is indicated by the `+` modifier.
In addition it is telling you that the patch on the remote has also changed
since you last requested review, indicated by the `!` modifier.

The `+` by itself is a pretty common modifier to see as it is there to simply
remind you that you have made changes to a patch and need to `sync` or
`request-review` for that patch again.

The `!` modifier is less common as it only happens when the patch on the
remote has changed since the last operation. This is generally because someone
either force pushed up a patch out of band of Git Patch Stack to replace that
commit or because someone added a commit to the branch that Git Patch Stack
created and associated to that patch. To resolve `!` modifiers you really need
to go look at the commits on the remote branch and see if there are any
changes there you want to keep. If so you should cherry-pick the ones you want
into your patch stack and squash/fix them up into the logical patch using the
`rebase` command. Then you should `sync` or `request-review` of that patch
again to get it back in sync.

The `↓` modifier indicates that the patch is conceptually behind. What this
means is that when the last rr/sync operation was performed the base of the
patch stack was at one point in the git tree but now it has progressed forward
as someone integrated changes into it. This can be addressed by doing a `gps
pull` to make sure that your local stack is up to date and integrates
everything from upstream and then doing a `gps sync` or `gps rr` to update the
remote with newly rebased patch.
")]
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
