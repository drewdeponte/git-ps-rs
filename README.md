# git-ps - Git Patch Stack

This is the official repository for the Rust implementation of `git-ps`. This
is in heavy development and does not have all the `git-ps` functionality
implemented yet. See the currently released `git-ps` implementation
[here](https://github.com/uptech/git-ps).

## Why a new implementation?

There are a number of reasons why long term it makes sense to implement a new
version of `git-ps` and more specifically implement it in Rust. However, the
biggest reason is performance. The current implementation is in Swift which
doesn't have decent bindings for libgit2. Therefore, we have to interacte with
git by spinning up subprocesses and capturing and parsing the output. This
results in the current implementation being extremely slow. Rust on the other
hand has fantastic libgit2 bindings and all the same great feature benefits
of Swift plus more.

- **portability** - the current stable release is implemented in Swift which
  isn't nearly as portable as Rust
- **performance** - in general Rust is decently faster than Swift, but it also
  has an ecosystem of libraries that facilitate making a significantly faster
  implementation
- **library ecosystem** - Rust provides a better ecosystem of libraries for
  this use case
- **safety** - Rust provides the same flexibility and language features as
  Swift but with memory safety guarantees
- **build dependencies** - the current stable version is implemented in Swift
  which requires Xcode as a build dependency which fundamentally means that we
  can only build it on macOS machines via Homebrew.
- **not a spike** - the current stable release was implemented in Swift and was
  implemented quick and dirty with no test coverage to simply validate the
  workflow of Git Patch Stack which we have done at this point. This
  implementation gives us the opportunity to actually design and implement it
  in a respectable fashion.
- **test coverage** - the current stable implementation has zero test coverage.
  This implementation gives us an opportunity to make sure we have test
  coverage from the get go.
- **community support** - with the stable implementations code base being a
  mess, it is hard form people to easily contribute to it.

## Status

Given that this project is currently under heavy development we are temporarily
releasing it under `gps` rather than `git-ps` until we reach feature
completion. This allows the current stable release of `git-ps` to be installed
as well as this version. Enabling users to use the new implementation prior
to it being feature complete.

The following is a breakdown of the planned commands and their
respective completion statuses.

* `ls` - under active development
* `rebase` - not started
* `pull` - not started
* `rr` - not started
* `pub` - not started
* `show` - not started
* `co` - not started

## Development

### Application Architecture

The following is a breakdown of the module hierarchy and what each of the
module's repsonsabilities are.

* `src`
	* `main` - command line parsing & sub command handoff
	* `ps` - parenting module collecting Patch Stack specific modules 
		* `utils` - generic utility functions
		* `git` - functionality for interfacing with git
		* `test` - test suite helpers
		* `ps` - generic shared patch stack functionality
		* `commands` - container module for the subcommand modules
			* `ls` - ls subcommand & supporting functionality
			* `pull` - pull subcommand & supporting functionality
			* `rebase` - rebase subcommand & supporting functionality
			* `rr` - rr subcommand & supporting functionality
			* `pub` - pub subcommand & supporting functionality
			* `show` - show subcommand & supporting functionality
			* `co` - co subcommand & supporting functionality

### Build

```text
cargo build
```

### Test

```text
cargo test
```

## License

`git-ps` is Copyright © 2020 UpTech Works, LLC. It is free software, and
may be redistributed under the terms specified in the LICENSE file.

## About <img src="http://upte.ch/img/logo.png" alt="uptech" height="48">

`git-ps` is maintained and funded by [UpTech Works, LLC][uptech], a software
design & development agency & consultancy.

We love open source software. See [our other projects][community] or
[hire us][hire] to design, develop, and grow your product.

[community]: https://github.com/uptech
[hire]: http://upte.ch
[uptech]: http://upte.ch