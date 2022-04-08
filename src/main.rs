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
    #[structopt(short = "n")]
    pub branch_name: Option<String>
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
pub enum Command {
    #[structopt(name = "branch", alias = "br", about = "(br) - Create a request review branch & cherry-pick the specified patch into it")]
    Branch(BranchCmdOpts),
    #[structopt(name = "integrate", alias = "int", about = "(int) - Integrate the specified patch into the patch stacks upstream remote")]
    Integrate(IntegrateCmdOpts),
    #[structopt(name = "list", alias = "ls", about = "(ls) - List the stack of patches and their associated state info")]
    List,
    #[structopt(name = "rebase", about = "Interactively rebase your stack of patches")]
    Rebase,
    #[structopt(name = "pull", about = "Pull changes down from upstream and rebase stack on top")]
    Pull,
    #[structopt(name = "request-review", alias = "rr", about = "(rr) - Request review of the specified patch")]
    RequestReview(RequestReview),
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
    };
}
