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

    // Generate Shell completions
    let _bash_completion_path =
        generate_to(Shell::Bash, &mut cmd, env!("CARGO_PKG_NAME"), &outdir)?;
    let _zsh_completion_path = generate_to(Shell::Zsh, &mut cmd, env!("CARGO_PKG_NAME"), &outdir)?;

    // println!("cargo:warning=completion file is generated: {_bash_completion_path:?}");
    // println!("cargo:warning=completion file is generated: {_zsh_completion_path:?}");

    // Generate primary man page
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    let _man_path = format!("{}/gps.1", outdir.to_str().unwrap());

    std::fs::write(_man_path.as_str(), buffer)?;
    // println!("cargo:warning=man file is generated: {_man_path:?}");

    // Generate subcommand man pages
    let name = "gps";

    for subcommand in cmd.get_subcommands() {
        let subcommand_name = subcommand.get_name();
        let subcommand_man_name = format!("{}-{}", name, &subcommand_name);
        let mut buffer: Vec<u8> = Default::default();
        let subcommand_man_name_clap_str_builder = clap::builder::Str::from(&subcommand_man_name);
        let cmd_clone: clap::Command = subcommand
            .clone()
            .name(subcommand_man_name_clap_str_builder);
        let man = clap_mangen::Man::new(cmd_clone);
        man.render(&mut buffer)?;
        let _man_path = format!("{}/{}.1", outdir.to_str().unwrap(), &subcommand_man_name);
        std::fs::write(_man_path.as_str(), buffer)?;
        // println!("cargo:warning=man file is generated: {_man_path:?}");
    }

    Ok(())
}
