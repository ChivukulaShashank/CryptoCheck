pub mod dataset;
pub mod models;
pub mod metrics;

pub use dataset::{Dataset, Sample};
pub use models::{Classifier, LogisticRegression, DecisionForest};
pub use metrics::{EvaluationResult, evaluate_predictions};

/// Trains a classifier on the training split and evaluates metrics on the test split
pub fn train_and_evaluate<C: Classifier>(
    mut model: C,
    dataset: &Dataset,
    train_ratio: f64,
) -> EvaluationResult {
    if dataset.len() < 10 {
        return EvaluationResult {
            model_name: model.name().to_string(),
            ..Default::default()
        };
    }

    let (train_set, test_set) = dataset.train_test_split(train_ratio);
    model.train(&train_set);

    let predictions: Vec<(f64, u8)> = test_set.samples
        .iter()
        .map(|sample| {
            let prob = model.predict_proba(&sample.features);
            (prob, sample.label)
        })
        .collect();

    evaluate_predictions(model.name(), &predictions)
}
