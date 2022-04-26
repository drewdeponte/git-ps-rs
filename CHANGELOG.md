# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Generated by [Git Changelog](https://github.com/uptech/git-cl), an open source project brought to you by [UpTech Works, LLC](https://upte.ch). A consultancy that partners with companies to help **build**, **launch**, and **refine** their products.


## Unreleased - now


## [3.2.0] - 2022-04-25

### Added
- support for GPG commit signing


## [3.1.1] - 2022-04-22

### Fixed
- bug where pull command was losing commits


## [3.1.0] - 2022-04-22

### Added
- bash & zsh completion scription generation to build process


## [3.0.1] - 2022-04-21

### Fixed
- error that was incorrectly being return about incorrect number of commits


## [3.0.0] - 2022-04-21

### Added
- request_review_post_sync hook support to request review command
- branch command as a bridge from patch stack to git concepts

### Changed
- ps::branch() function name to ps::request_review_branch()
- `branch` command name to `request-review-branch`

### Fixed
- bug `int -f` would incorrectly fail out on patches without a ps-id

### Removed
- the `branch` commands shortname `br`


## [2.0.0] - 2022-04-18

[3.2.0]: https://github.com/uptech/git-ps-rs/compare/92a0e9e...584ae9e
[3.1.1]: https://github.com/uptech/git-ps-rs/compare/6be7401...92a0e9e
[3.1.0]: https://github.com/uptech/git-ps-rs/compare/3bdfadd...6be7401
[3.0.1]: https://github.com/uptech/git-ps-rs/compare/9e0b101...3bdfadd
[3.0.0]: https://github.com/uptech/git-ps-rs/compare/6e800b2...9e0b101
[2.0.0]: https://github.com/uptech/git-ps-rs/compare/ae31eb6...6e800b2