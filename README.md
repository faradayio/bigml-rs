# bigml client for Rust (WORK IN PROGRESS)

[![Documentation](https://img.shields.io/badge/documentation-docs.rs-yellow.svg)](https://docs.rs/bigml/)

An interface to the [BigML][] machine learning API, written in Rust.  This
requires nightly Rust, because it depends on `#[feature(proc_macro)]` (so
that we can mix serde and regular macros).

What works:

- Uploading sources that are small enough to fit in memory.
- Executing scripts and getting the output values.

It's pretty easy to add new types and fields.  See `src/resources` for
existing examples.

[BigML]: https://bigml.com/
