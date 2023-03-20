use std::env;
use std::fs;
use std::process;
use structopt::clap::Shell;

include!("src/cli.rs");

fn main() {
    // OUT_DIR is set by Cargo and it's where any additional build artifacts
    // are written.
    let outdir = match env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => {
            eprintln!(
                "OUT_DIR environment variable not defined. \
        Please file a bug: \
        https://github.com/uptech/git-ps-rs/issues/new"
            );
            process::exit(1);
        }
    };
    fs::create_dir_all(&outdir).unwrap();

    ApplicationArguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, &outdir);
    ApplicationArguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, &outdir);
}
