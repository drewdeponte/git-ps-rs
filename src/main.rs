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
mod cli;

fn main() {
    let opt = cli::ApplicationArguments::from_args();

    match opt.command {
        cli::Command::Branch(opts) => commands::branch::branch(opts.start_patch_index, opts.end_patch_index, opts.branch_name),
        cli::Command::RequestReviewBranch(opts) => commands::request_review_branch::request_review_branch(opts.patch_index, opts.branch_name),
        cli::Command::Integrate(opts) => commands::integrate::integrate(opts.patch_index, opts.force, opts.keep_branch, opts.branch_name),
        cli::Command::List => commands::list::list(opt.color),
        cli::Command::Rebase(opts) => commands::rebase::rebase(opts.r#continue),
        cli::Command::Pull => commands::pull::pull(opt.color),
        cli::Command::RequestReview(opts) => commands::request_review::request_review(opts.patch_index, opts.branch_name),
        cli::Command::Show(opts) => commands::show::show(opts.patch_index),
        cli::Command::Sync(opts) => commands::sync::sync(opts.patch_index, opts.branch_name),
        cli::Command::Isolate(opts) => commands::isolate::isolate(opts.patch_index),
        cli::Command::Checkout(opts) => commands::checkout::checkout(opts.patch_index),
        cli::Command::CreatePatch => commands::create_patch::create_patch(),
        cli::Command::AmendPatch => commands::amend_patch::amend_patch(),
        cli::Command::Status => commands::status::status(),
        cli::Command::Add(opts) => commands::add_changes_to_stage::add_changes_to_stage(opts.interactive, opts.patch, opts.edit, opts.all, opts.files),
        cli::Command::Log => commands::log::log(),
        cli::Command::Unstage(opts) => commands::unstage::unstage(opts.files)
    };
}
