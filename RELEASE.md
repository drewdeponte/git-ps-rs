# Release Process

## Cut Release in Git Repository

- create a version bump commit and integrate it
	- update the version in the `Cargo.toml`
	- run `cargo build` so that the `Cargo.lock` is update
	- `gps c` to create the commit
	- `gps int` the version bump patch
	- `gps pull` to make sure you have the latest state
- copy the SHA of the integrated version bump commit, `git log`
- tag the version bump commit with the version, e.g. `git tag 6.6.0 <coppied-sha>`
- push tags up to remote, `git push --tags`
- update `CHANGELOG.md` for new release
	- `git-cl full > CHANGELOG.md`
	- stage the changes to `CHANGELOG.md`, e.g. `git add CHANGELOG.md`
	- commit changes to `CHANGELOG.md`, e.g. `gps c`
	- integrate changes to `CHANGELOG.md`, `gps int`

## Publish release to Homebrew

- copy the SHA of the integrated version bump commit, `git log`
- update the `Formula/git-ps-rs.rb` file to set the tag to the new tag value, and update the SHA to the copied sha
- stage, commit, and push the change indicating `Release vX.Y.Z of git-ps-rs`

## Publish release to Crates.io

- `cargo publish --dry-run`
- `cargo publish`

## Update GitHub Releases

This is important because it is utilized by the in-app versioning check that
happens when a `gps pull` operation occurs. So publishing the release here is
what will make people get notified while using `gps` that there is a newer
version available.

- go to [GitHub Releases Page](https://github.com/uptech/git-ps-rs/releases)
- verify that everything looks as you expect prior to publishing a new release
- click the [Draft a new release](https://github.com/uptech/git-ps-rs/releases/new) button
- choose the tag you pushed up in the previous section
- enter the tag name in the "Release title" field, e.g. `6.6.0`
- copy the changes from this version in the `CHANGELOG.md` and paste them into the "Describe this release" text area
- click the "Publish release" button

## Notify Package Maintainers

Notify package maintainers of new release so that they can update the packages
they maintain on the respective platforms.

- NixOS: Ali Caglayan (slack: `@Ali Caglayan`)
- Homebrew: Drew De Ponte (slack: `@Drew`)
- Crates.io: Drew De Ponte (slack: `@Drew`)
