use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Debug, Args)]
pub struct RequestReview {
    pub patch_index_or_range_batch: String,
    /// Use the provided branch name instead of generating one
    #[arg(short = 'n')]
    pub branch_name: Option<String>,
    /// disable isolation verification hook
    #[arg(long = "no-isolation-verification-hook", action(ArgAction::SetFalse))]
    pub isolation_verification_hook: bool,
    /// disable post sync hook
    #[arg(long = "no-post-sync-hook", action(ArgAction::SetFalse))]
    pub post_sync_hook: bool,
}

#[derive(Debug, Args)]
pub struct BranchCmdOpts {
    pub patch_index_or_range: String,
    /// Use the provided branch name instead of generating one
    #[arg(short = 'n')]
    pub branch_name: Option<String>,
}

#[derive(Debug, Args)]
pub struct IntegrateCmdOpts {
    pub patch_index_or_range: String,
    /// Use the provided branch name instead of generating one
    #[arg(short = 'n')]
    pub branch_name: Option<String>,
    /// Keep request-review branches around (a.k.a. don't clean up request
    /// review branches)
    #[arg(short = 'k', long = "keep-branch")]
    pub keep_branch: bool,
    /// Skip safety checks and publish
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ShowCmdOpts {
    pub patch_index_or_range: String,
}

#[derive(Debug, Args)]
pub struct IsolateCmdOpts {
    pub patch_index_or_range: Option<String>,
}

#[derive(Debug, Args)]
pub struct CheckoutCmdOpts {
    pub patch_index: usize,
}

#[derive(Debug, Args)]
pub struct RebaseCmdOpts {
    /// continue a rebase that was paused
    #[arg(long = "continue")]
    pub r#continue: bool,
}

#[derive(Debug, Args)]
pub struct BackupStackCmdOpts {
    pub branch_name: String,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// (b) - Create a branch for a patch or patch series
    #[command(name = "branch", alias = "b")]
    Branch(BranchCmdOpts),
    /// (int) - Integrate the specified patch into the patch stacks upstream
    /// remote
    #[command(name = "integrate", alias = "int")]
    Integrate(IntegrateCmdOpts),

    /// (ls) - List the stack of patches and their associated state info
    #[command(
        name = "list",
        alias = "ls",
        long_about = r"
(ls) - List the stack of patches and their associated state info

The `list` command lists out your stack of patches in a format that exposes
the patch index on the far left followed by the short sha of the git
commit, followed by the patch summary, and finally followed by the state
information.

[index] [sha] [summary (50 chars)         ]  ( [status] )

The patch index value is used with other commands, e.g. `gps show
<patch-index>`.

State information exists between a patch in the patch stack and a branch.
As you use Git Patch Stack your patches will be associated with one or
more local branches and each of those branches will likely have a remote
tracking branch associated to them.

So we represent state with two main prefixes, l & r.

l    - indicating that the following state indicators are between the local
       branch & patch in the patch stack
r    - indicating that the following state indicators are between the
       remote branch & patch in the patch stack

The presence of these prefixes also communicates the existance of a local
or remote branch in association with the patch. So if you saw a state
of ( ) it would indicate that the patch has no local branches & has no
remote branches.

Each of these prefixes are paired with zero or more of the following state
indicators.

*    - the patch in the respective branch has a different diff than
       the patch in the patch stack
!    - the respective branch has one or more non-patch commits in it

The following are some simple examples of state indications so you can
start to understand.

( )       - patch has no local & no remote branches associated
( l )     - patch has a local branch associated & the diff match
( lr )    - patch has a local branch & remote branch associated & the
            diffs match
( l*r* )  - patch has a local branch & remote branch associated, but the
            diffs don't match
( l*r*! ) - patch has a local branch & remote branch associated, but the
            diffs don't match & the remote has a non-patch commit in it

In the most common case you will have a single branch pairing
(local & remote) associated with your patches and you will see the patch
state simply represented as above.

However, Git Patch Stack supports having multiple branch pairings associated
with a patch and it also supports custom naming of branches if you don't want
to use a generated branch name. This is especially useful when creating a
patch series.

If a patch falls into either of these scenarios the state will be presented
in long form where the branch name is provided in front of each state
indication. So each branch will have its branch name appear followed by its
associated state indication.

[branch-name] [state indications]

These pairings of branch name and state indications are simply chained
together with spaces. So for example, if we assume we have a patch that
is associated with two branches, foo & bar. The patch state information
might look something like the following.

( foo lr bar l*r* )

In the above state information example we can see that there are 4 branches
that exist with the patch in them. Specifically, there is a local branch
named `foo` and it has a remote tracking branch that also has the patch in it.
We can see that because there is no `*` or `!` characters after the `l` or `r`,
associated with the `foo` branch, we know that the patch diffs all match.

We can also see that the patch exists in another local branch named `bar`, as
well as the remote tracking branch of `bar`. The `*` characters here indicate
that both the copy of the patch in both the local `bar` branch and the remote
tracking branch of `bar` have different diffs than the patch in the patch
stack.
"
    )]
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
    #[command(name = "rebase")]
    Rebase(RebaseCmdOpts),

    /// Pull changes down from upstream and rebase stack on top
    #[command(name = "pull")]
    Pull,
    /// (rr) - Request review of the specified patch
    #[command(name = "request-review", alias = "rr")]
    RequestReview(RequestReview),
    /// Show the identified patch in raw form
    #[command(name = "show")]
    Show(ShowCmdOpts),

    /// (iso) - isolate a patch or series of patches for manual testing or evaluation.
    ///
    /// The `isolate` command isolates a patch or series of patches for manual testing or
    /// evaluation by making sure no uncommitted changes exist, creating a temporary branch based
    /// on the patch stacks base, cherry-picking the patch(es) to it, and then checking out that
    /// branch.
    ///
    /// If you have the `isolate_post_checkout` hook setup then that will be executed after
    /// successfully checking out the branch.
    ///
    /// When you are done manually testing or evaluating the patch(es) in isolation you can return
    /// to the stack that you were on when you switched into isolation mode by running `gps iso`.
    #[command(name = "isolate", alias = "iso")]
    Isolate(IsolateCmdOpts),
    /// (co) - Checkout the patch identified by the patch-index, leaving you
    /// in a headless state.
    #[command(name = "checkout", alias = "co")]
    Checkout(CheckoutCmdOpts),

    /// (f) - Fetch state from remote and display upstream patches
    #[command(name = "fetch", alias = "f")]
    Fetch,

    /// (bs) backup your current patch stack to the given branch name
    #[cfg(feature = "backup_cmd")]
    #[command(name = "backup-stack", alias = "bs")]
    BackupStack(BackupStackCmdOpts),
}

#[derive(Debug, Parser)]
#[command(
    name = "gps",
    author,
    version,
    long_about = r"
Git Patch Stack has two main categories of commands, CORE & ANCILLARY.

The CORE commands make up the basis of all the core functionality. These include the following.

pull, list, rebase, request-review, integrate

The ANCILLARY commands are simply there in support of the other commands. These include the
following.

fetch, branch, checkout, show, isolate

So, if you are just getting started. Starting with the CORE commands will probably help.
"
)]
pub struct Cli {
    /// disable color output
    #[arg(long = "no-color", global = true, action(ArgAction::SetFalse))]
    pub color: bool,

    #[command(subcommand)]
    pub command: Command,
}
