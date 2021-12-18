//! A WhizzML script on BigML.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use super::id::*;
use super::library::Library;
use super::status::*;
use super::{Resource, ResourceCommon};
use crate::errors::*;

/// A WhizzML script on BigML.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "script"]
#[non_exhaustive]
pub struct Script {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Script>,

    /// The status of this resource.
    pub status: GenericStatus,

    /// The source code of this script.
    pub source_code: String,
}

/// Arguments used to create a new BigML script.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct Args {
    /// The category code which best describes this script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<i64>,

    /// A human-readable description of this script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A list of "library/..." identifiers to import.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<Id<Library>>,

    /// A list of script input declarations.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,

    /// A human-readable name for this script.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A list of script output declarations.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<Output>,

    /// The source code of this script.
    pub source_code: String,

    /// User-defined tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Args {
    /// Create a new `Args` value.
    pub fn new<S: Into<String>>(source_code: S) -> Args {
        Args {
            category: Default::default(),
            description: Default::default(),
            imports: Default::default(),
            inputs: Default::default(),
            name: Default::default(),
            outputs: Default::default(),
            source_code: source_code.into(),
            tags: Default::default(),
        }
    }
}

impl super::Args for Args {
    type Resource = Script;
}

/// A script input declaration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Input {
    /// The variable name of this input.
    pub name: String,
    /// The type of this input.
    #[serde(rename = "type")]
    pub type_: Type,
    /// The default value of this input.
    pub default: Option<serde_json::Value>,
    /// A description of this input.
    pub description: Option<String>,
}

impl Input {
    /// Create a new `Input` value.
    pub fn new<S: Into<String>>(name: S, type_: Type) -> Input {
        Input {
            name: name.into(),
            type_,
            default: None,
            description: None,
        }
    }
}

/// A script output declaration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Output {
    /// The variable name of this output.
    pub name: String,
    /// The type of this output.
    #[serde(rename = "type")]
    pub type_: Type,
    /// A description of this output.
    pub description: Option<String>,
}

impl Output {
    /// Create a new `Output` value.
    pub fn new<S: Into<String>>(name: S, type_: Type) -> Output {
        Output {
            name: name.into(),
            type_,
            description: None,
        }
    }
}

/// Helper macro to declare `Type`.
macro_rules! declare_type_enum {
    ($($name:ident => $api_name:expr,)+) => (
        /// Input or output type.
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Serialize, PartialEq)]
        #[allow(missing_docs)]
        #[non_exhaustive]
        pub enum Type {
            $(
                #[serde(rename = $api_name)]
                $name,
            )+
        }

        impl fmt::Display for Type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match *self {
                    $( Type::$name => $api_name.fmt(f), )*
                }
            }
        }

        impl FromStr for Type {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self> {
                match s {
                    $( $api_name => Ok(Type::$name), )*
                    _ => {
                        Err(Error::UnknownBigMlType { type_name: s.to_owned() })
                    }
                }
            }
        }
    )
}

declare_type_enum! {
    String => "string",
    Categorical => "categorical",
    Text => "text",
    Items => "items",
    Number => "number",
    // DateTime => "date-time",
    Numeric => "numeric",
    Integer => "integer",
    Boolean => "boolean",
    List => "list",
    Map => "map",
    ListOfString => "list-of-string",
    ListOfInteger => "list-of-integer",
    ListOfNumber => "list-of-number",
    ListOfMap => "list-of-map",
    ListOfBoolean => "list-of-boolean",
    ResourceId => "resource-id",
    SupervisedModelId => "supervised-model-id",
    ProjectId => "project-id",
    SourceId => "source-id",
    DatasetId => "dataset-id",
    SampleId => "sample-id",
    ModelId => "model-id",
    EnsembleId => "ensemble-id",
    LogisticRegressionId => "logisticregression-id",
    DeepnetId => "deepnet-id",
    TimeseriesId => "timeseries-id",
    PredictionId => "prediction-id",
    BatchPredictionId => "batchprediction-id",
    EvaluationId => "evaluation-id",
    AnomalyId => "anomaly-id",
    AnomalyScoreId => "anomalyscore-id",
    BatchAnomolayScoreId => "batchanomalyscore-id",
    ClusterId => "cluster-id",
    CentroidId => "centroid-id",
    BatchCentroidId => "batchcentroid-id",
    AssociationId => "association-id",
    AssociationSetId => "associationset-id",
    TopicModelId => "topicmodel-id",
    TopicDistributionId => "topicdistribution-id",
    BatchTopicDistribution => "batchtopicdistribution-id",
    CorrelationId => "correlation-id",
    StatisticalTestId => "statisticaltest-id",
    LibraryId => "library-id",
    ScriptId => "script-id",
    ExecutionId => "execution-id",
    Configuration => "configuration-id",
}

#[test]
fn parse_type() {
    let ty: Type = "categorical".parse().unwrap();
    assert_eq!(ty, Type::Categorical);
}

#[test]
fn display_type() {
    assert_eq!(format!("{}", Type::Categorical), "categorical");
}
