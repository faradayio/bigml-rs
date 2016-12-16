//! An evaluation of how well a model (or ensemble) predicts the data.

use serde::{Serialize, Deserialize};
use std::fmt;

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "evaluation";

    /// An evaluation of how well a model (or ensemble) predicts the data.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize, Clone)]
    pub struct Evaluation<R: Result> {
        /// The status of this resource.
        pub status: GenericStatus,

        /// The result of this evaluation.
        pub result: R,
    }
}

/// The result of an evaluation.
///
/// TODO: I'm not sure we want to shadow `Result`.  But this name will
/// basically always be qualified, so maybe it's OK.
pub trait Result: fmt::Debug + Deserialize + Serialize + Sized {
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

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
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

// TODO: RegressionResult.
