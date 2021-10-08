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
        Command::List => ps::commands::ls::ls()
    };

}
