// This is the the main application module. In module namespacing it is the
// `crate` module. It is generally responsible for housing the main() entry
// point. In our case we have the main entry point responsible for the
// following:
//
// - declaring the CLI options interface & help messaging
// - parsing the CLI options into a data structure (ApplicationArguments)
// - map CLI options data structure to subcommand calls & arguments
//
// So any code that fits the above responsibilities should live within this
// module.

use structopt::StructOpt;

mod commands;

#[derive(Debug, StructOpt)]
pub struct RequestReview {
    pub patch_index: usize,
    // #[structopt(short = "n")]
    // pub branch_name: Option<String>
}

#[derive(Debug, StructOpt)]
pub struct BranchCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub struct IntegrateCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub struct ShowCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub struct SyncCmdOpts {
  pub patch_index: usize
}

#[derive(Debug, StructOpt)]
pub struct IsolateCmdOpts {
  pub patch_index: Option<usize>
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// (br) - Create a request review branch & cherry-pick the specified patch into it
    #[structopt(name = "branch", alias = "br")]
    Branch(BranchCmdOpts),
    /// (int) - Integrate the specified patch into the patch stacks upstream remote
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
    /// (iso) - Isolate a patch by creating a temporary branch based on upstream, cherry-picking the patch to it, & checking it out
    #[structopt(name = "isolate", alias = "iso")]
    Isolate(IsolateCmdOpts)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "git-ps")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command
}

fn main() {
    let opt = ApplicationArguments::from_args();

    match opt.command {
        Command::Branch(opts) => commands::plumbing::branch::branch(opts.patch_index),
        Command::Integrate(opts) => commands::porcelain::integrate::integrate(opts.patch_index),
        Command::List => commands::porcelain::ls::ls(),
        Command::Rebase => commands::porcelain::rebase::rebase(),
        Command::Pull => commands::porcelain::pull::pull(),
        Command::RequestReview(opts) => commands::porcelain::rr::rr(opts.patch_index),
        Command::Show(opts) => commands::porcelain::show::show(opts.patch_index),
        Command::Sync(opts) => commands::plumbing::sync::sync(opts.patch_index),
        Command::Isolate(opts) => commands::porcelain::isolate::isolate(opts.patch_index),
    };
}
