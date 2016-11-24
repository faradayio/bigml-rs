// Included directly into client.rs after pre-processing by serde.

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::result;
use std::str::FromStr;
use serde_json;

use errors::*;

//-------------------------------------------------------------------------
// ResourceStatus interfaces

/// A BigML status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatusCode {
    /// BigML is waiting on another resource before processing this one.
    Waiting,
    /// The processing job has been added to the queue.
    Queued,
    /// Actual processing has started.
    Started,
    /// Part of the job has been performed.
    InProgress,
    /// Summary statistics for a dataset are available.
    Summarized,
    /// The resource is ready.
    Finished,
    /// Something went wrong processing the task.
    Faulty,
    /// Something has gone wrong in BigML, perhaps an outage.
    Unknown,
}

impl ResourceStatusCode {
    /// Is BigML still working on ingesting and processing this resource?
    pub fn is_working(self) -> bool {
        use ResourceStatusCode::*;
        match self {
            Waiting | Queued | Started | InProgress | Summarized => true,
            _ => false,
        }
    }

    /// Has BigML successfully finished processing this resource?
    pub fn is_ready(self) -> bool {
        self == ResourceStatusCode::Finished
    }

    /// Did something go wrong while processing this resource?
    pub fn is_err(self) -> bool {
        self == ResourceStatusCode::Faulty ||
            self == ResourceStatusCode::Unknown
    }
}

impl Deserialize for ResourceStatusCode {
    fn deserialize<D>(deserializer: &mut D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        match i64::deserialize(deserializer)? {
            0 => Ok(ResourceStatusCode::Waiting),
            1 => Ok(ResourceStatusCode::Queued),
            2 => Ok(ResourceStatusCode::Started),
            3 => Ok(ResourceStatusCode::InProgress),
            4 => Ok(ResourceStatusCode::Summarized),
            5 => Ok(ResourceStatusCode::Finished),
            -1 => Ok(ResourceStatusCode::Faulty),
            -2 => Ok(ResourceStatusCode::Unknown),
            code => {
                let msg = format!("Unknown BigML resource status code {}", code);
                Err(<D::Error as serde::Error>::invalid_value(&msg))
            }
        }
    }
}

/// Status of a resource.  BigML actually defines many different "status"
/// types, one for each resource, but quite a few of them have are highly
/// similar.  This interface tries to generalize over the most common
/// versions.
pub trait ResourceStatus {
    /// Status code.
    fn code(&self) -> ResourceStatusCode;

    /// Human-readable status message.
    fn message(&self) -> &str;

    /// Number of milliseconds which were needed to create this resource.
    fn elapsed(&self) -> Option<u64>;

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    fn progress(&self) -> Option<f32>;
}

/// Status of a generic resource.
#[derive(Debug, Deserialize)]
pub struct GenericResourceStatus {
    /// Status code.
    pub code: ResourceStatusCode,

    /// Human-readable status message.
    pub message: String,

    /// Number of milliseconds which were needed to create this resource.
    pub elapsed: Option<u64>,

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    pub progress: Option<f32>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl ResourceStatus for GenericResourceStatus {
    fn code(&self) -> ResourceStatusCode {
        self.code
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn elapsed(&self) -> Option<u64> {
        self.elapsed
    }

    fn progress(&self) -> Option<f32> {
        self.progress
    }
}

//-------------------------------------------------------------------------
// ResourceProperties interfaces

/// This trait allows access to common properties shared by all resource
/// types.
pub trait ResourceProperties: fmt::Debug + Deserialize {
    /// The status code for this resource.
    fn status(&self) -> &ResourceStatus;
}

/// A trait representing a BigML data type.  Caution!  This is a very
/// abstract trait and implementations are not expected to carry any actual
/// data.  Rather, this mostly exists to be used as a "tag" and to create
/// associations between related types.
pub trait Resource {
    /// The properties of resources of this type.
    type Properties: ResourceProperties;

    /// The prefix used for all IDs of this type.
    fn id_prefix() -> &'static str;
}

//-------------------------------------------------------------------------
// ResourceId

/// A strongly-typed "resource ID" used to identify many different kinds of
/// BigML resources.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId<R: Resource> {
    /// The ID of the resource.
    id: String,
    /// A special 0-byte field which exists just to mention the type `R`
    /// inside the struct, and thus avoid compiler errors about unused type
    /// parameters.
    _phantom: PhantomData<R>,
}

impl<R: Resource> ResourceId<R> {
    /// Get this resource as a string.
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl<R: Resource> FromStr for ResourceId<R> {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        if !id.starts_with(R::id_prefix()) {
            Ok(ResourceId {
                id: id.to_owned(),
                _phantom: PhantomData,
            })
        } else {
            Err(ErrorKind::WrongResourceType(R::id_prefix(), id.to_owned()).into())
        }
    }
}

impl<R: Resource> fmt::Debug for ResourceId<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> fmt::Display for ResourceId<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> Deserialize for ResourceId<R> {
    fn deserialize<D>(deserializer: &mut D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        let id: String = String::deserialize(deserializer)?;
        if !id.starts_with(R::id_prefix()) {
            Ok(ResourceId {
                id: id,
                _phantom: PhantomData,
            })
        } else {
            let err: Error =
                ErrorKind::WrongResourceType(R::id_prefix(), id).into();
            Err(<D::Error as serde::Error>::invalid_value(&format!("{}", err)))
        }
    }
}

impl<R: Resource> Serialize for ResourceId<R> {
    fn serialize<S>(&self, serializer: &mut S) -> result::Result<(), S::Error>
        where S: Serializer
    {
        self.id.serialize(serializer)
    }
}

//-------------------------------------------------------------------------
// Model type

/// BigML supports two main kinds of models: "classification" models, which
/// are used to predict category properties, and "regression" models, which
/// are used to predict numeric properties.  These models are treated
/// differently in many places.
///
/// `ModelType` is a "marker" trait that we use to keep track of which kind
/// of model we're working with.  It allows us to figure out which related
/// types are associated with each model.
///
/// We inherit from the trait `fmt::Debug`, not because anybody should ever
/// create or print a `ModelType`, but because we're used as type parameter
/// to many structs which `#[derive(Debug)]`, which won't work unless all
/// type parameters are themselves `fmt::Debug`, even if they're not needed
/// to print the struct.
pub trait ModelType: fmt::Debug {
    /// The results of an evaluation of this model.
    type EvaluationResult: fmt::Debug + Deserialize + Serialize + 'static;
}

/// Classification models are used to predict category properties.
#[derive(Debug)]
pub struct ClassificationModel;

impl ModelType for ClassificationModel {
    type EvaluationResult = ClassificationEvaluationResult;
}

// TODO: RegressionModel and RegressionEvaluationResult.

//-------------------------------------------------------------------------
// Ensemble

/// A group of many related models.
pub struct Ensemble;

impl Resource for Ensemble {
    type Properties = EnsembleProperties;

    fn id_prefix() -> &'static str {
        "execution/"
    }
}

/// Properties of an ensemble resource.
///
/// TODO: Still lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct EnsembleProperties {
    /// Used to classify by industry or category.  0 is "Miscellaneous".
    pub category: i64,

    /// An HTTP status code, typically either 201 or 200.
    ///
    /// TODO: Deserialize as a `reqwest::StatusCode`?
    pub code: u16,

    // The dataset used to create this ensemble.
    //pub dataset: ResourceId<Dataset>,

    /// Text describing this source.  May contain limited Markdown.
    pub description: String,

    /// The name of this execution.
    pub name: String,

    /// The ID of this execution.
    pub resource: ResourceId<Execution>,

    /// The current status of this execution.
    pub status: GenericResourceStatus,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl ResourceProperties for EnsembleProperties {
    fn status(&self) -> &ResourceStatus {
        &self.status
    }
}


//-------------------------------------------------------------------------
// Evaluation

/// An execution of a WhizzML script.
pub struct Evaluation<M: ModelType> {
    /// A special 0-byte field which exists just to mention the type `M`
    /// inside the struct, and thus avoid compiler errors about unused type
    /// parameters.
    _phantom: PhantomData<M>,
}

impl<M: ModelType> Resource for Evaluation<M> {
    type Properties = EvaluationProperties<M>;

    fn id_prefix() -> &'static str {
        "evaluation/"
    }
}

/// Properties of a BigML evaluation.
///
/// TODO: Still lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct EvaluationProperties<M: ModelType> {
    /// The result of this evaluation.
    pub result: M::EvaluationResult,

    /// The status of this resource.
    pub status: GenericResourceStatus,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl<M: ModelType> ResourceProperties for EvaluationProperties<M> {
    fn status(&self) -> &ResourceStatus {
        &self.status
    }
}

/// The result of an evaluation.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassificationEvaluationResult {
    /// The names of our classifications.
    pub class_names: Vec<String>,

    /// According to BigML, "Measures the performance of the classifier
    /// that predicts the mode class for all the instances in the dataset."
    pub mode: DetailedClassificationEvaluationResult,

    /// The performance of this model.
    pub model: DetailedClassificationEvaluationResult,

    /// According to BigML, "Measures the performance of the classifier
    /// that predicts a random class for all the instances in the dataset."
    pub random: DetailedClassificationEvaluationResult,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

/// The detailed result of an evaluation using specific criteria.
#[derive(Debug, Deserialize, Serialize)]
pub struct DetailedClassificationEvaluationResult {
    /// The portion of instances we classified correctly.
    pub accuracy: f64,
    /// The average f-measure over all classes.
    pub average_f_measure: f64,
    /// The average phi over all classes.
    pub average_phi: f64,
    /// The average precision over all classes.
    pub average_precision: f64,
    /// The average recall over all classes.
    pub average_recall: f64,
    /// A list of rows of the confusion matrix for this model.
    pub confusion_matrix: Vec<Vec<f64>>,
    /// Statistics for each of the individidual classes.
    pub per_class_statistics: Vec<ClassificationPerClassStatistics>,
    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

/// The detailed result of an evaluation using specific criteria.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassificationPerClassStatistics {
    /// The portion of instances in this class that were correctly
    /// classified.
    pub accuracy: f64,
    /// The the of this class.
    pub class_name: String,
    /// The harmonic mean of precision and recall.
    pub f_measure: f64,
    /// See
    /// [Wikipedia](http://en.wikipedia.org/wiki/Matthews_correlation_coefficient).
    pub phi_coefficient: f64,
    /// The fraction of positives that were true positives. (TP / (TP + FP))
    pub precision: f64,
    /// The number of true positives over the number of actual positives in
    /// the dataset. (TP / (TP + FN))
    pub recall: f64,
}

//-------------------------------------------------------------------------
// Executions

/// An execution of a WhizzML script.
pub struct Execution;

impl Resource for Execution {
    type Properties = ExecutionProperties;

    fn id_prefix() -> &'static str {
        "execution/"
    }
}

/// Properties of a BigML source.
///
/// TODO: Still lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct ExecutionProperties {
    /// Used to classify by industry or category.  0 is "Miscellaneous".
    pub category: i64,

    /// An HTTP status code, typically either 201 or 200.
    ///
    /// TODO: Deserialize as a `reqwest::StatusCode`?
    pub code: u16,

    /// Text describing this source.  May contain limited Markdown.
    pub description: String,

    /// Further information about this execution.
    pub execution: ExecutionData,

    /// The name of this execution.
    pub name: String,

    /// The ID of this execution.
    pub resource: ResourceId<Execution>,

    // The script executed.
    //pub script: ResourceId<Script>,

    /// The current status of this execution.
    pub status: GenericResourceStatus,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl ResourceProperties for ExecutionProperties {
    fn status(&self) -> &ResourceStatus {
        &self.status
    }
}

/// Data about a script execution.
///
/// TODO: Lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct ExecutionData {
    /// Names, values and types of resources output by the script.
    outputs: Vec<(String, serde_json::Value, String)>,

    /// Result values from the script.
    result: Vec<serde_json::Value>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}



//-------------------------------------------------------------------------
// Sources

/// A data source used by BigML.
pub struct Source;

impl Resource for Source {
    type Properties = SourceProperties;

    fn id_prefix() -> &'static str {
        "source/"
    }
}

/// Properties of BigML source.
///
/// TODO: Still lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct SourceProperties {
    /// Used to classify data by industry or category.  0 is
    /// "Miscellaneous".
    pub category: i64,

    /// An HTTP status code, typically either 201 or 200.
    ///
    /// TODO: Deserialize as a `reqwest::StatusCode`?
    pub code: u16,

    /// The number of credits it cost to create this source.
    pub credits: f64,

    /// Text describing this source.  May contain limited Markdown.
    pub description: String,

    /// The name of the file uploaded.
    pub file_name: String,

    /// An MD5 hash of the uploaded file.
    pub md5: String,

    /// The name of this data source.
    pub name: String,

    /// The identifier for this source.
    pub resource: ResourceId<Source>,

    /// The number of bytes of the source.
    pub size: u64,

    /// The status of this source.
    pub status: GenericResourceStatus,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl ResourceProperties for SourceProperties {
    fn status(&self) -> &ResourceStatus {
        &self.status
    }
}
