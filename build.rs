use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
  ApplicationArguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, "target");
  ApplicationArguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
}
