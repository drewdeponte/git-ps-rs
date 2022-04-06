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
  mess, it is hard for people to easily contribute to it.

## Status

Given that this project is currently under heavy development we are temporarily
releasing it under `gps` rather than `git-ps` until we reach feature
completion. This allows the current stable release of `git-ps` to be installed
as well as this version. Enabling users to use the new implementation prior
to it being feature complete.

The following is a breakdown of the planned commands and their
respective completion statuses.

* `ls` - functional (missing status info, needs cleanup & tests)
* `rebase` - functional (needs some cleanup & tests)
* `pull` - functional (needs cleanup & tests)
* `rr` - not started
* `pub` - not started
* `show` - not started
* `co` - not started

## Development


### Modules

Below are outlined two different views of the module hierarchy of this app. The
hope is that with the combination of these two views you will understand the
architecture of the application and where things live/should live to ease the
process of getting acclimated to a new code base.

#### Filesystem Hierarchy

The following is a breakdown of the filesystem hierarchy and what each of the
module's is responsible for.

* `src`
	* `main` - command line entry point, parsing & sub command handoff
	* `commands` - module collecting plumbing & porcelain cmd funcs for cli
		* `plumbing` - module of plumbing command funcs for cli
			* `branch` - branch command & supporting functionality
		* `porcelain` - module of porcelain command funcs for cli
			* `ls` - ls command & supporting functionality
			* `pull` - pull command & supporting functionality
			* `rebase` - rebase command & supporting functionality
			* `rr` - rr command & supporting functionality 
			* `pub` - pub command & supporting functionality
			* `show` - show command & supporting functionality
			* `co` - co command & supporting functionality
	* `lib` - library entry point
	* `ps` - parenting module collecting Patch Stack specific modules 
		* `utils` - generic utility functions
		* `git` - functionality for interfacing with git
		* `test` - test suite helpers
		* `ps` - generic shared patch stack functionality
		* `ls` - ls command function in the library
		* `pull` - pull command function in the library
		* `rebase` - rebase command function in the library
		* `rr` - rr command function in the library
		* `branch` - branch command function in the library

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

## About <img src="https://uploads-ssl.webflow.com/6222b1faf83d05669ca63972/6222b3714a050d0e8f8bd2ef_logo-color.svg" alt="uptech" height="48">

`git-ps` is maintained and funded by [UpTech Works, LLC][uptech], a software
design & development agency & consultancy.

We love open source software. See [our other projects][community] or
[hire us][hire] to design, develop, and grow your product.

[community]: https://github.com/uptech
[hire]: http://upte.ch
[uptech]: http://upte.ch
