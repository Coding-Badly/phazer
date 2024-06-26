# Changelog

## phazer 0.2.0 (2024-06-16)
[v0.1.2...v0.2.0](https://github.com/Coding-Badly/phazer/compare/v0.1.2...v0.2.0)

### Added

- `CommitStrategy` allows customizing the Phazer commit strategy.  Two options are included: `SimpleRenameStrategy` and `RenameWithRetryStrategy`.
- `PhazerBuilder` introduced for building a non-default Phazer.
- `Phazer::commit2` method returns an error / self tuple when something goes wrong to help with error recovery.
- `test_helpers` feature was added to aid with testing.
- `no-writer-commit-works` test was added to ensure committing without writing returns success.
- `one-wins-in-race` test was added to ensure committing on multiple threads works.
- `posix-read-only-works` test was added to ensure read-only targets can be replaced under POSIX operating systems.
- `write-commit-works` ensures every phase is correct for commit.
- `write-no-commit-works` ensures every phase is correct for rollback.
- A few examples.

### Changed

- Phazer is now tokio::spawn friendly.
- The working file is now named {stem}.{ext}.phazer-working-{process_id}-{phazer_id}
- Common test code refactored into the tests/common module.
- All tests run using GitHub Actions for Linux, macOS, and Windows
- Significant improvements to the documentation

### Removed

- All things "bug-001".  The bug is fixed.  The change has been merged into main.
- Miri tests.

## phazer 0.1.2 (2023-11-26)

### Added

- Initial release of the `tokio` (asynchronous) file creation helper (Phazer).

## phazer 0.1.1

### Added

- Initial release of the `simple` (synchronous) file creation helper (Phazer).

## phazer 0.1.0

### Added

- Initial release: https://github.com/Coding-Badly/phazer
