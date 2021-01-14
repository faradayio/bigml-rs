# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.7.0 - 2021-01-14

### Changed

- Update to `bytes` 1.0, `reqwest` 0.11, `tokio` 1.0.1 and `tokio-util` 0.6.1. Several of these libraries define types that are visible in our public API, so this is technically a semver change. But other than the version updates, no APIs have changed.

## 0.6.6 - 2020-07-31

### Changed

- `Client::wait` now uses exponential backoff for up to 10 total minutes, because we're seeing a lot more temporary infrastructure errors from BigML.
- 500, 503 and 504 HTTP errors are now temporary.

## 0.6.5 - 2020-05-07

### Added

- `bigml`: Added `Error::original_bigml_error` to unwrap our own error type.

### Fixed

- `bigml`: We now have a correctly implemented and documented contract for `Client::wait` and `Client::wait_opt`'s error reporting.
- `bigml-parallel`: We should now honor `--retry-on`.

## 0.6.4 - 2020-05-06

### Added

- `bigml-parallel`: Added new `--retry-on` and `--retry-count` arguments that can be used to retry failed executions.

### Fixed

- `bigml-parallel`: Removed `.timeout()` clauses that were probably unnecessary, because the code in question never returned `WaitStatus::Waiting`. This might slightly change retry behavior.
- Fixed lots of minor warnings from the newest `clippy` and Rust releases.
