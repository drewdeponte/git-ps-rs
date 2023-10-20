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
        cli::Command::Branch(opts) => commands::branch::branch(
            opts.start_patch_index,
            opts.end_patch_index,
            opts.branch_name,
            opts.push_to_remote,
            cli.color,
        ),
        cli::Command::RequestReviewBranch(opts) => {
            commands::request_review_branch::request_review_branch(
                opts.patch_index_or_range,
                opts.branch_name,
            )
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
            opts.patch_index_or_range,
            opts.branch_name,
            cli.color,
        ),
        cli::Command::BatchRequestReview(opts) => {
            commands::batch_request_review::batch_request_review(opts.patch_index, cli.color)
        }
        cli::Command::Show(opts) => commands::show::show(opts.patch_index_or_range),
        cli::Command::Sync(opts) => {
            commands::sync::sync(opts.patch_index_or_range, opts.branch_name)
        }
        cli::Command::Isolate(opts) => {
            commands::isolate::isolate(opts.patch_index_or_range, cli.color)
        }
        cli::Command::Checkout(opts) => commands::checkout::checkout(opts.patch_index),
        cli::Command::CreatePatch => commands::create_patch::create_patch(),
        cli::Command::AmendPatch(opts) => commands::amend_patch::amend_patch(opts.no_edit),
        cli::Command::Status => commands::status::status(),
        cli::Command::Log => commands::log::log(),
        cli::Command::Fetch => commands::fetch::fetch(cli.color),
        #[cfg(feature = "backup_cmd")]
        cli::Command::BackupStack(opts) => commands::backup_stack::backup_stack(opts.branch_name),
    };
}
