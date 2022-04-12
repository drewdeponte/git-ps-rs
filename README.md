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

* `list - ls` - doneish
	* doesn't include change detection since last sync
* `rebase` - done
* `pull` - done
* `branch - br` - doneish
	* doesn't support custom branch naming, `-n` option
	* doesn't support patch series
* `sync` - doneish
	* doesn't support custom branch naming, `-n` option
	* doesn't support patch series
* `request-review - rr` - doneish
	* doesn't support custom branch naming, `-n` option
	* doesn't support patch series
	* doesn't support hook
* `integrate - int` - doneish
	* doesn't support custom branch naming, `-n` option
	* doesn't support request review verification check & bypass `-f` option)
	* doesn't support patch series
* `show` - done
* `checkout - co` - done
* `isolate - iso` - doneish
	* but no way to get back to previous branch other than, `git checkout main`

## Development

To find details on contributing and developing this project see [DEVELOPMENT.md](DEVELOPMENT.md)

## License

`git-ps` is Copyright © 2020 UpTech Works, LLC. It is free software, and
may be redistributed under the terms specified in the LICENSE file.

## About <img src="https://uploads-ssl.webflow.com/6222b1faf83d05669ca63972/624dc2dea4bbe5dd1d21a04c_uptechstudio-logo.svg" alt="Uptech Studio">

`git-ps-rs` is maintained and funded by [Uptech Studio][uptech], a software
design & development studio.

We love open source software. See [our other projects][community] or
[hire us][hire] to design, develop, and grow your product.

[community]: https://github.com/uptech
[hire]: https://www.uptechstudio.com/careers
[uptech]: https://uptechstudio.com
