//! Input arguments for BigML execution resources.

use anyhow::{format_err, Error, Result};
use log::warn;
use serde_json::Value;
use std::str::FromStr;

/// An input argument for a BigML execution resource.
#[derive(Debug)]
pub struct ExecutionInput {
    /// The name of this input.
    pub name: String,

    /// The JSON value of this input.
    pub value: Value,
}

/// Declare a `FromStr` implementation for `Input` so that `structopt` can parse
/// command-line arguments directly into `Input` values.
impl FromStr for ExecutionInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let split = s.splitn(2, '=').collect::<Vec<&str>>();
        if split.len() != 2 {
            return Err(format_err!("input {:?} must have form \"key=value\"", s,));
        }
        let name = split[0].to_owned();
        let value = match serde_json::from_str(split[1]) {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    "could not parse input {:?} as JSON (treating as string): {}",
                    s, err,
                );
                Value::String(split[1].to_owned())
            }
        };
        Ok(ExecutionInput { name, value })
    }
}

#[test]
fn parses_json_values() {
    let examples = &[
        ("x=null", Value::Null),
        ("x=true", Value::Bool(true)),
        ("x=false", Value::Bool(false)),
        ("x=0", Value::Number(0.into())),
        ("x=[true]", Value::Array(vec![Value::Bool(true)])),
        ("x=\"hi\"", Value::String("hi".to_owned())),
    ];
    for (input, expected) in examples {
        let parsed = input.parse::<ExecutionInput>().unwrap();
        assert_eq!(&parsed.value, expected);
    }
}

#[test]
fn defaults_to_string_values() {
    let parsed = "x=hi".parse::<ExecutionInput>().unwrap();
    assert_eq!(parsed.value, Value::String("hi".to_owned()));
}
