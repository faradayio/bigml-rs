//! An evaluation of how well a model (or ensemble) predicts the data.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;

use super::{Resource, ResourceCommon};
use super::id::*;
use super::status::*;

/// An evaluation of how well a model (or ensemble) predicts the data.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[serde(bound(deserialize = ""))]
#[api_name = "evaluation"]
pub struct Evaluation<R: Result> {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Evaluation<R>>,

    /// The status of this resource.
    pub status: GenericStatus,

    /// The result of this evaluation.
    pub result: R,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// The result of an evaluation.
///
/// TODO: I'm not sure we want to shadow `Result`.  But this name will
/// basically always be qualified, so maybe it's OK.
pub trait Result: fmt::Debug + DeserializeOwned + Serialize + Sized + 'static {
}

/// The result of evaluating a classifier.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClassificationResult {
    /// The names of our classifications.
    pub class_names: Vec<String>,

    /// According to BigML, "Measures the performance of the classifier
    /// that predicts the mode class for all the instances in the dataset."
    pub mode: DetailedClassificationResult,

    /// The performance of this model.
    pub model: DetailedClassificationResult,

    /// According to BigML, "Measures the performance of the classifier
    /// that predicts a random class for all the instances in the dataset."
    pub random: DetailedClassificationResult,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

impl Result for ClassificationResult {}

/// The detailed result of an evaluation using specific criteria.
#[derive(Debug, Deserialize, Serialize)]
pub struct DetailedClassificationResult {
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
    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
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
    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

// TODO: RegressionResult.
