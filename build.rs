use std::env;
use std::fs;
use std::io::Error;
use std::process;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
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

    let mut cmd = Cli::command();

    let _bash_completion_path =
        generate_to(Shell::Bash, &mut cmd, env!("CARGO_PKG_NAME"), &outdir)?;
    let _zsh_completion_path = generate_to(Shell::Zsh, &mut cmd, env!("CARGO_PKG_NAME"), &outdir)?;

    // println!("cargo:warning=completion file is generated: {_bash_completion_path:?}");
    // println!("cargo:warning=completion file is generated: {_zsh_completion_path:?}");

    Ok(())
}
