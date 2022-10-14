# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2022-10-14

- Update to latest versions of dependencies and latest Rust compiler. Should be backwards compatible with 0.9.2.
- Set official maintenance status to "as-is". Issues that affect Faraday or `dbcrossbar` will still be addressed. No guarantees about anything else.

## [0.9.2] - 2022-05-19

### Fixed

- Allow SourceLocation to be missing.

## [0.9.1] - 2021-12-18

### Fixed

- Fix doc test that was breaking builds.

## [0.9.0] - 2021-12-18

### Changed

- Our `Error` has been modernized. It implements `std::error::Error`, it names the source field `source`, and its zero-argument variants are now empty structs, to leave room for adding backtraces later.
- We now log using `tracing` instead of `log`, for much better logging of complex async code.

## [0.8.1] - 2021-12-16

### Added

- We now build official binaries for Mac M1 systems.

### Changed

- Our release `*.zip`-file naming convention has changed to include the full target description (OS, CPU, etc) instead of just `linux` or `osx`.

## [0.8.0] - 2021-11-17

### Changed

- Switch from OpenSSL to RusTLS using `rustls-tls-native-roots`. This is a breaking change, but we try to use the OS cert store like before. This _might_ cause issues if you're using a private BigML cluster. Please report any problems.

### Fixed

- Fixed a number of security advisories by updating dependencies. (This should not affect users of `bigml` who were updating their own `Cargo.lock` files. They should already have been up to date.)

## [0.7.0] - 2021-01-14

### Changed

- Update to `bytes` 1.0, `reqwest` 0.11, `tokio` 1.0.1 and `tokio-util` 0.6.1. Several of these libraries define types that are visible in our public API, so this is technically a semver change. But other than the version updates, no APIs have changed.

## [0.6.6] - 2020-07-31

### Changed

- `Client::wait` now uses exponential backoff for up to 10 total minutes, because we're seeing a lot more temporary infrastructure errors from BigML.
- 500, 503 and 504 HTTP errors are now temporary.

## [0.6.5] - 2020-05-07

### Added

- `bigml`: Added `Error::original_bigml_error` to unwrap our own error type.

### Fixed

- `bigml`: We now have a correctly implemented and documented contract for `Client::wait` and `Client::wait_opt`'s error reporting.
- `bigml-parallel`: We should now honor `--retry-on`.

## [0.6.4] - 2020-05-06

### Added

- `bigml-parallel`: Added new `--retry-on` and `--retry-count` arguments that can be used to retry failed executions.

### Fixed

- `bigml-parallel`: Removed `.timeout()` clauses that were probably unnecessary, because the code in question never returned `WaitStatus::Waiting`. This might slightly change retry behavior.
- Fixed lots of minor warnings from the newest `clippy` and Rust releases.
