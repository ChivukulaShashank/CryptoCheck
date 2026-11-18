use super::dataset::Dataset;
use rand::{Rng, seq::SliceRandom};

pub trait Classifier: Send + Sync {
    fn name(&self) -> &'static str;
    fn train(&mut self, dataset: &Dataset);
    fn predict_proba(&self, features: &[f64]) -> f64;
}

/// Regularized Logistic Regression trained with Stochastic Gradient Descent (SGD)
#[derive(Debug, Clone)]
pub struct LogisticRegression {
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
    epochs: usize,
    l2_reg: f64,
}

impl Default for LogisticRegression {
    fn default() -> Self {
        Self {
            weights: Vec::new(),
            bias: 0.0,
            learning_rate: 0.08,
            epochs: 45,
            l2_reg: 0.001,
        }
    }
}

impl LogisticRegression {
    pub fn new(epochs: usize, lr: f64) -> Self {
        Self {
            epochs,
            learning_rate: lr,
            ..Default::default()
        }
    }

    fn sigmoid(z: f64) -> f64 {
        1.0 / (1.0 + (-z.clamp(-30.0, 30.0)).exp())
    }
}

impl Classifier for LogisticRegression {
    fn name(&self) -> &'static str {
        "Logistic Regression (SGD)"
    }

    fn train(&mut self, dataset: &Dataset) {
        if dataset.is_empty() {
            return;
        }

        let num_features = dataset.samples[0].features.len();
        self.weights = vec![0.0; num_features];
        self.bias = 0.0;

        let n = dataset.samples.len() as f64;
        let mut rng = rand::thread_rng();

        for _ in 0..self.epochs {
            // Shuffle training samples
            let mut indices: Vec<usize> = (0..dataset.samples.len()).collect();
            indices.shuffle(&mut rng);

            for idx in indices {
                let sample = &dataset.samples[idx];
                let prob = self.predict_proba(&sample.features);
                let target = sample.label as f64;
                let error = prob - target;

                // Gradient update
                for j in 0..num_features {
                    let grad = error * sample.features[j] + self.l2_reg * self.weights[j];
                    self.weights[j] -= self.learning_rate * grad / n;
                }
                self.bias -= self.learning_rate * error / n;
            }
        }
    }

    fn predict_proba(&self, features: &[f64]) -> f64 {
        if self.weights.is_empty() || features.is_empty() {
            return 0.5;
        }

        let z: f64 = self.weights.iter()
            .zip(features.iter())
            .map(|(w, x)| w * x)
            .sum::<f64>() + self.bias;

        Self::sigmoid(z)
    }
}

/// Single Decision Stump for Ensemble
#[derive(Debug, Clone)]
struct DecisionStump {
    feature_idx: usize,
    threshold: f64,
    polarity: i8, // 1: > threshold is class 1, -1: <= threshold is class 1
}

impl DecisionStump {
    fn predict(&self, features: &[f64]) -> f64 {
        let val = features.get(self.feature_idx).copied().unwrap_or(0.0);
        if (self.polarity == 1 && val > self.threshold) || (self.polarity == -1 && val <= self.threshold) {
            1.0
        } else {
            0.0
        }
    }
}

/// Random Forest Ensemble Classifier with Bagging
#[derive(Debug, Clone)]
pub struct DecisionForest {
    trees: Vec<DecisionStump>,
    num_trees: usize,
}

impl Default for DecisionForest {
    fn default() -> Self {
        Self {
            trees: Vec::new(),
            num_trees: 20,
        }
    }
}

impl DecisionForest {
    pub fn new(num_trees: usize) -> Self {
        Self {
            trees: Vec::new(),
            num_trees,
        }
    }
}

impl Classifier for DecisionForest {
    fn name(&self) -> &'static str {
        "Random Forest Distinguisher"
    }

    fn train(&mut self, dataset: &Dataset) {
        if dataset.is_empty() {
            return;
        }

        self.trees.clear();
        let mut rng = rand::thread_rng();
        let n_samples = dataset.samples.len();
        let n_features = dataset.samples[0].features.len();

        for _ in 0..self.num_trees {
            // Bootstrap sample
            let mut bootstrap = Vec::with_capacity(n_samples);
            for _ in 0..n_samples {
                let idx = rng.gen_range(0..n_samples);
                bootstrap.push(&dataset.samples[idx]);
            }

            // Find best split on a random feature subset
            let f_idx = rng.gen_range(0..n_features);
            let mut best_threshold = 0.0;
            let mut best_gini = f64::MAX;
            let mut best_polarity = 1;

            // Sample candidate thresholds
            for _ in 0..15 {
                let candidate_idx = rng.gen_range(0..bootstrap.len());
                let threshold = bootstrap[candidate_idx].features[f_idx];

                for polarity in [1, -1] {
                    let mut left_c0 = 0;
                    let mut left_c1 = 0;
                    let mut right_c0 = 0;
                    let mut right_c1 = 0;

                    for s in &bootstrap {
                        let val = s.features[f_idx];
                        let is_class1 = (polarity == 1 && val > threshold) || (polarity == -1 && val <= threshold);
                        if is_class1 {
                            if s.label == 1 { left_c1 += 1; } else { left_c0 += 1; }
                        } else {
                            if s.label == 1 { right_c1 += 1; } else { right_c0 += 1; }
                        }
                    }

                    let left_total = (left_c0 + left_c1) as f64;
                    let right_total = (right_c0 + right_c1) as f64;

                    let gini_left = if left_total > 0.0 {
                        1.0 - (left_c0 as f64 / left_total).powi(2) - (left_c1 as f64 / left_total).powi(2)
                    } else { 0.5 };

                    let gini_right = if right_total > 0.0 {
                        1.0 - (right_c0 as f64 / right_total).powi(2) - (right_c1 as f64 / right_total).powi(2)
                    } else { 0.5 };

                    let weighted_gini = (left_total * gini_left + right_total * gini_right) / (left_total + right_total).max(1.0);

                    if weighted_gini < best_gini {
                        best_gini = weighted_gini;
                        best_threshold = threshold;
                        best_polarity = polarity;
                    }
                }
            }

            self.trees.push(DecisionStump {
                feature_idx: f_idx,
                threshold: best_threshold,
                polarity: best_polarity,
            });
        }
    }

    fn predict_proba(&self, features: &[f64]) -> f64 {
        if self.trees.is_empty() {
            return 0.5;
        }

        let votes: f64 = self.trees.iter().map(|t| t.predict(features)).sum();
        votes / self.trees.len() as f64
    }
}
