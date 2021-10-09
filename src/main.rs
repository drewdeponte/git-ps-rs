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

pub mod ps;

// #[derive(Debug, StructOpt)]
// pub struct Show {
//     pub patch_index: u32
// }

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "ls")]
    List,
    #[structopt(name = "rebase")]
    Rebase,
    #[structopt(name = "pull")]
    Pull,
    // Show(Show)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "git-ps")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command
}

fn main() {
    let opt = ApplicationArguments::from_args();
    // println!("{:?}", opt);

    match opt.command {
        Command::List => ps::commands::ls::ls(),
        Command::Rebase => ps::commands::rebase::rebase(),
        Command::Pull => ps::commands::pull::pull()
    };
}
