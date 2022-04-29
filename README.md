# git-ps-rs - Git Patch Stack

This is the official source repository for the Git Patch Stack command line
interface, `gps`. 

The legacy command line tool for Git Patch Stack,
[git-ps](https://github.com/uptech/git-ps) is no longer active and is only
arround for historical reasons.

## What is Git Patch Stack?

Git Patch Stack is a software development **methodology**, [Git][]
**workflow**, and a **command line tool** to help make working in this manner
as easy as possible.

It is focused around the idea of creating and managing a stack of individual
logically chunked patches through the development & review workflow while still
integrating with the peer review tools & platforms people are already
comfortable with.

### The Methodology

First and foremost Git Patch Stack is a software development methodology
centralized around facilitating valuable [pre-commit][] code reviews while
gaining as many of the benefits of the [Continuous Integration Methodology][]
as possible. If you interested in how & why we came to this methodology please
check out our article, [Journey to Small Pull Requests][].

### The Git Workflow

Given that [Git][] is the central tool we use for source control management, it
is important to make sure that we use it in a manner that facilitates the goals
we are trying to accomplish. This is where Git Patch Stack as a Git Workflow
comes into play. To gain a deeper understanding of Git Patch Stack as a Git
Workflow please checkout our article, [How we should be using Git][].

### The Command Line Tool

The Git Patch Stack command line tool is an extension of [Git][] designed to
make creating & managing your stacks of patches throughout the development and
review workflow as easy as possible and enabling you to think in terms of the
concepts of the Git Patch Stack methodology rather than the concepts of
[Git][].

## Operation Summary

The following are a quick summary of the main commands that enable the Git
Patch Stack workflow.

- `pull` - pull changes down from upstream & rebase the stack on top
- `list (ls)` - list your patches in the stack you are on and their states
- `rebase` - interactively rebase your stack of patches
- `request-review (rr)` - request review of a patch (Requires a [hook](#hooks) to be set up.)
- `integrate (int)` - integrate the specified patch into the patch stack's upstream remote

You can get a full breakdown of all the commands by running

```
gps --help
```

You can also get detailed help about specific commands by use the `--help` switch with the command. Example:

```
gps request-review --help
```

## Installation 

If you are on a platform other than macOS you will have to build your own
version from source.

#### **NOTE: In order to use `gps request-review`you must set up a [hook](#hooks) after installation.**

### macOS

To install on macOS we provide a [Homebrew][] tap which provides the
`git-ps-rs` formula. You can use it by doing the following:

#### Add the Tap

	brew tap "uptech/homebrew-oss"

#### brew install

	brew install uptech/oss/git-ps-rs

#### zsh & bash Completions

Our [Homebrew][] formulae installs the zsh & bash completion scripts into the
standard [Homebrew][] shell completions location. So you just need to make sure
that path is configured in your shell configuration. For zsh it is generally
something like the following:

```zsh
fpath=(/opt/homebrew/share/zsh/site-functions/ $fpath)
autoload -Uz compinit
compinit
```

### Build from Source

If you are on another platform you will have to build from source. Given
that `git-ps-rs` is managed via [Cargo][]. It can be built as follows:

	cargo build --release

Once you have built it successfully you can use the `mv` command to move the
`target/release/gps` file into `/usr/local/bin/` or some other location in your
`PATH` environment variable.

#### zsh & bash completions

The zsh and bash completion scripts are generated as part of the build process
by [Cargo][]'s custom build script, `build.rs` at the root of the project.

The scripts are output to the [Cargo -
OUT_DIR](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
location, generally `target/release/build/gps-*/out` where the `*` is a hash
value. The files are named as follows.

- `gps.bash` - bash completion script
- `_gps` - zsh completion script

## Hooks

Git Patch Stack takes the stance that it shouldn't be bound to a specific
source control management platform (GitHub, Bitbucket, GitLab, etc.) or a
particular request review process. Even across projects.

To give our users this flexibility we have created a **hooks** system for the
`request-review` command allowing the users to configure & customize what the
`request-review` command does.

A hook is simply an executable file (script, binary, etc.) that is named
according to the particular hook name and location in one of the two general
locations for hooks.

- `.git/git-ps/hooks/` - repository specific hooks
- `~/.config/git-ps/hooks/` - user global hooks

Repository specific hooks are searched for first, and if not found then
it searches in the user's global hooks. This allows users to have a sane default
configuration globally while also being able to configure specific repositories
with different hooks.

The following is a list of currently supported hooks (their expected filenames).

- `request_review_post_sync` - hook executed by `request-review` command after succesfully syncing the patch to remote - generally used to create a pull request / patch email & send it
- `isolate_post_checkout` - hook executed by `isolate` command after successfully creating the temporary branch, cherry-picking the patch to it, and checking the branch out

You can find examples of hooks that you can straight up use or just use as a starting point in [example_hooks](/example_hooks).

Here is an [example hook](/example_hooks/request_review_post_sync-github-cli-example) that uses [GitHub CLI][] to create a pull request.

### Setup Global Hook Directory

Make sure that the Global Hook Directory is created with the following:

```
mkdir -p ~/.config/git-ps/hooks
```

Copy the example hook of your choice to the Global Hooks Directory and give it execute permissions. The following is an example.

```
curl -fsSL https://raw.githubusercontent.com/uptech/git-ps-rs/main/example_hooks/request_review_post_sync-github-cli-example --output ~/.config/git-ps/hooks/request_review_post_sync
chmod u+x ~/.config/git-ps/hooks/request_review_post_sync
```

## Configs

Git Patch Stack supports three layers of configuration files that can all have
the same configuration options in them.

- `~/.config/git-ps/config.toml` - user global preferences (useful for setting sane defaults for repos without configs)
- `repo_root/.git/git-ps/config.toml` - personal preferences specific to the repository
- `repo_root/.git-ps/config.toml` - communal preferences specific to the repository (useful for enforcing certain preferences for all devs of a repo)

The **communal repo config** takes precedence over the **repo specific personal config** which takes precedence over the **user global config**.

The following is an example of config format with the default values. *Note:* All sections and settings are optional.

```
[pull]
show_list_post_pull = false # list patches after successful pull

[request_review]
verify_isolation = true     # run isolation check & any isolation hooks

[integrate]
verify_isolation = true	    # run isolation check & any isolation hooks
require_verification = true # require user to approve integration
```

## Product

To find details on the concept of the product and questions & answers in that space see [PRODUCT.md](PRODUCT.md).

## Development

To find details on contributing and developing this project see [DEVELOPMENT.md](DEVELOPMENT.md)

## License

`git-ps-rs` is Copyright © 2020 UpTech Works, LLC. It is free software, and
may be redistributed under the terms specified in the LICENSE file.

## About <img src="https://uploads-ssl.webflow.com/6222b1faf83d05669ca63972/624dc2dea4bbe5dd1d21a04c_uptechstudio-logo.svg" alt="Uptech Studio">

`git-ps-rs` is maintained and funded by [Uptech Studio][uptech], a software
design & development studio.

We love open source software. See [our other projects][community] or
[hire us][hire] to design, develop, and grow your product.

[community]: https://github.com/uptech
[hire]: https://www.uptechstudio.com/careers
[uptech]: https://uptechstudio.com
[Cargo]: https://doc.rust-lang.org/cargo/
[Homebrew]: https://brew.sh
[Git]: https://git-scm.com
[Continuous Integration Methodology]: https://en.wikipedia.org/wiki/Continuous_integration
[pre-commit]: https://www.devart.com/review-assistant/learnmore/pre-commit-vs-post-commit.html
[Journey to Small Pull Requests]: https://engineering.uptechstudio.com/blog/journey-to-small-pull-requests/
[How we should be using Git]: https://engineering.uptechstudio.com/blog/how-we-should-be-using-git/
[GitHub CLI]: https://cli.github.com
