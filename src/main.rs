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

mod cli;
mod commands;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Branch(opts) => {
            commands::branch::branch(opts.patch_index_or_range, opts.branch_name, cli.color)
        }
        cli::Command::Integrate(opts) => commands::integrate::integrate(
            opts.patch_index_or_range,
            opts.force,
            opts.keep_branch,
            opts.branch_name,
            cli.color,
        ),
        cli::Command::List => commands::list::list(cli.color),
        cli::Command::Rebase(opts) => commands::rebase::rebase(opts.r#continue),
        cli::Command::Pull => commands::pull::pull(cli.color),
        cli::Command::RequestReview(opts) => commands::request_review::request_review(
            opts.patch_index_or_range_batch,
            opts.branch_name,
            cli.color,
            opts.isolation_verification_hook,
            opts.post_sync_hook,
        ),
        cli::Command::Sha(opts) => {
            commands::sha::sha(opts.patch_index, cli.color, opts.exclude_newline)
        }
        cli::Command::Id => commands::id::id(cli.color),
        cli::Command::Show(opts) => commands::show::show(opts.patch_index_or_range),
        cli::Command::Append(opts) => {
            commands::append::append(opts.patch_index_or_range, opts.branch_name, cli.color)
        }
        cli::Command::Push(opts) => commands::push::push(opts.branch_name, cli.color),
        cli::Command::Isolate(opts) => {
            commands::isolate::isolate(opts.patch_index_or_range, cli.color)
        }
        cli::Command::Checkout(opts) => commands::checkout::checkout(opts.patch_index),
        cli::Command::Fetch => commands::fetch::fetch(cli.color),
        #[cfg(feature = "backup_cmd")]
        cli::Command::BackupStack(opts) => commands::backup_stack::backup_stack(opts.branch_name),
    };
}
