use serde::{Serialize, Deserialize};

/// Comprehensive ML classification and side-channel leakage evaluation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub model_name: String,
    pub total_samples: usize,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub roc_auc: f64,
    /// Confusion Matrix: [[TN, FP], [FN, TP]]
    pub confusion_matrix: [[usize; 2]; 2],
    /// Estimated Shannon mutual information / side-channel leakage extracted (in bits)
    pub leakage_bits: f64,
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self {
            model_name: "Uninitialized".to_string(),
            total_samples: 0,
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            roc_auc: 0.5,
            confusion_matrix: [[0, 0], [0, 0]],
            leakage_bits: 0.0,
        }
    }
}

/// Evaluates binary classification predictions
pub fn evaluate_predictions(
    model_name: &str,
    predictions: &[(f64, u8)], // (predicted_probability, ground_truth)
) -> EvaluationResult {
    if predictions.is_empty() {
        return EvaluationResult {
            model_name: model_name.to_string(),
            ..Default::default()
        };
    }

    let mut tn = 0;
    let mut fp = 0;
    let mut fn_count = 0;
    let mut tp = 0;

    for &(prob, actual) in predictions {
        let predicted_class = if prob >= 0.5 { 1 } else { 0 };
        match (actual, predicted_class) {
            (0, 0) => tn += 1,
            (0, 1) => fp += 1,
            (1, 0) => fn_count += 1,
            (1, 1) => tp += 1,
            _ => (),
        }
    }

    let total = predictions.len() as f64;
    let accuracy = (tp + tn) as f64 / total;
    let precision = if tp + fp > 0 { tp as f64 / (tp + fp) as f64 } else { 0.0 };
    let recall = if tp + fn_count > 0 { tp as f64 / (tp + fn_count) as f64 } else { 0.0 };
    let f1_score = if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    // Calculate ROC-AUC via trapezoidal rule
    let roc_auc = compute_roc_auc(predictions);

    // Mutual Information estimate: I(X; Y) = H(Y) - H(Y|X)
    let p_y1 = (tp + fn_count) as f64 / total;
    let p_y0 = 1.0 - p_y1;
    let entropy_y = -(if p_y1 > 0.0 { p_y1 * p_y1.log2() } else { 0.0 }
        + if p_y0 > 0.0 { p_y0 * p_y0.log2() } else { 0.0 });

    // Conditional entropy bound from classifier error rate e
    let error_rate = (1.0 - accuracy).clamp(1e-6, 0.999999);
    let conditional_entropy = -(error_rate * error_rate.log2() + (1.0 - error_rate) * (1.0 - error_rate).log2());
    let leakage_bits = (entropy_y - conditional_entropy).max(0.0);

    EvaluationResult {
        model_name: model_name.to_string(),
        total_samples: predictions.len(),
        accuracy,
        precision,
        recall,
        f1_score,
        roc_auc,
        confusion_matrix: [[tn, fp], [fn_count, tp]],
        leakage_bits,
    }
}

/// Area Under the Receiver Operating Characteristic Curve (ROC-AUC)
fn compute_roc_auc(predictions: &[(f64, u8)]) -> f64 {
    let mut sorted = predictions.to_vec();
    // Sort descending by predicted probability
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let positives = sorted.iter().filter(|&&(_, actual)| actual == 1).count();
    let negatives = sorted.len() - positives;

    if positives == 0 || negatives == 0 {
        return 0.5; // Undefined / uninformative
    }

    let mut tp = 0;
    let mut fp = 0;
    let mut prev_fpr = 0.0;
    let mut prev_tpr = 0.0;
    let mut auc = 0.0;

    for &(_, actual) in &sorted {
        if actual == 1 {
            tp += 1;
        } else {
            fp += 1;
        }

        let tpr = tp as f64 / positives as f64;
        let fpr = fp as f64 / negatives as f64;

        // Trapezoidal area accumulation
        auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0;
        prev_fpr = fpr;
        prev_tpr = tpr;
    }

    auc.clamp(0.0, 1.0)
}
