# A unofficial, incomplete `bigml` client for Rust

[![Documentation](https://img.shields.io/badge/documentation-docs.rs-yellow.svg)](https://docs.rs/bigml/)

An interface to the [BigML][] machine learning API, written in Rust. We use this at Faraday, so it's pretty reliable for what it does. But it omits many features that we don't need to access from Rust. In particular, we focus first on supporting WhizzML scripts, and many other parts of the API are much less complete.

What works:

- Fetching information about many different kinds of resources.
- Creating a few kinds of resources.
- Updating selected properties of a few kinds of resources.
- Uploading sources that are small enough to fit in memory.
- Executing scripts and getting the output values.

It's pretty easy to add new types and fields.  See `src/resources` for existing examples.

## Maintenance status

This project has been moved to the "back burner." Issues that affect Faraday or `dbcrossbar` will still be addressed. No guarantees about anything else. Please contact the authors if you want to make heavy use of BigML from Rust.

The `wait` and retry logic may be spun out into a separate crate at some point.

[BigML]: https://bigml.com/
