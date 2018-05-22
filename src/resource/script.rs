//! A WhizzML script on BigML.

use serde_json;
use std::{fmt, str::FromStr};

use errors::*;
use super::Resource;
use super::id::*;
use super::library::Library;
use super::status::*;

resource! {
    api_name "script";

    /// A WhizzML script on BigML.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Script {
        /// The status of this resource.
        pub status: GenericStatus,

        /// The source code of this script.
        pub source_code: String,
    }
}

/// Arguments used to create a new BigML script.
#[derive(Debug, Serialize)]
pub struct Args {
    /// The category code which best describes this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub category: Option<i64>,

    /// A human-readable description of this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,

    /// A list of "library/..." identifiers to import.
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub imports: Vec<Id<Library>>,

    /// A list of script input declarations.
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub inputs: Vec<Input>,

    /// A human-readable name for this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub name: Option<String>,

    /// A list of script output declarations.
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub outputs: Vec<Output>,

    /// The source code of this script.
    pub source_code: String,

    /// User-defined tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl Args {
    /// Create a new `ScriptNew` value.
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
            _hidden: (),
        }
    }
}

impl super::Args for Args {
    type Resource = Script;
}

/// A script input declaration.
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// Placeholder to allow extensibility without breaking the API.
    #[serde(default, skip_serializing)]
    _placeholder: (),
}

impl Input {
    /// Create a new `Input` value.
    pub fn new<S: Into<String>>(name: S, type_: Type) -> Input {
        Input {
            name: name.into(),
            type_: type_,
            default: None,
            description: None,
            _placeholder: (),
        }
    }
}

/// A script output declaration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Output {
    /// The variable name of this output.
    pub name: String,
    /// The type of this output.
    #[serde(rename = "type")]
    pub type_: Type,
    /// A description of this output.
    pub description: Option<String>,
    /// Placeholder to allow extensibility without breaking API.
    #[serde(default, skip_serializing)]
    _placeholder: (),
}

impl Output {
    /// Create a new `Output` value.
    pub fn new<S: Into<String>>(name: S, type_: Type) -> Output {
        Output {
            name: name.into(),
            type_: type_,
            description: None,
            _placeholder: (),
        }
    }
}

/// Helper macro to declare `Type`.
macro_rules! declare_type_enum {
    ($($name:ident => $api_name:expr,)+) => (
        /// Input or output type.
        #[derive(Clone, Copy, Debug, Deserialize, Eq, Serialize, PartialEq)]
        #[allow(missing_docs)]
        pub enum Type {
            $(
                #[serde(rename = $api_name)]
                $name,
            )+
        }

        impl fmt::Display for Type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
                        Err(format_err!("Unknown BigML type: {:?}", s).into())
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
