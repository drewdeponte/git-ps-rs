## Product

Location for Product oriented questions and answers to help people get up to
speed on the product side of things.

### Why a new implementation?

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
