use crate::crypto::CryptoTrace;

/// ML Feature Vector and Binary Label
#[derive(Debug, Clone)]
pub struct Sample {
    pub features: Vec<f64>,
    pub label: u8, // 0 = Fixed / Baseline, 1 = Secret-dependent / Target
}

/// Dataset container with preprocessing utilities
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub samples: Vec<Sample>,
}

impl Dataset {
    pub fn new() -> Self {
        Self { samples: Vec::new() }
    }

    /// Extracts features from crypto traces:
    /// Feature 0: Normalized cycle count
    /// Feature 1: Deviation from baseline mean
    /// Feature 2: High-order cycle dispersion / pulse energy
    pub fn from_traces(traces: &[CryptoTrace]) -> Self {
        if traces.is_empty() {
            return Self::new();
        }

        let mean_cycles: f64 = traces.iter().map(|t| t.cycles as f64).sum::<f64>() / traces.len() as f64;
        let std_dev: f64 = (traces.iter().map(|t| {
            let d = t.cycles as f64 - mean_cycles;
            d * d
        }).sum::<f64>() / traces.len() as f64).sqrt().max(1.0);

        let samples = traces.iter().map(|t| {
            let raw = t.cycles as f64;
            let norm_cycles = (raw - mean_cycles) / std_dev;
            let dev = raw - mean_cycles;
            let pulse_energy = norm_cycles.powi(2);

            Sample {
                features: vec![norm_cycles, dev, pulse_energy],
                label: t.class,
            }
        }).collect();

        Self { samples }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Splits dataset into (Train, Test) with a given train ratio (e.g. 0.75)
    pub fn train_test_split(&self, train_ratio: f64) -> (Dataset, Dataset) {
        let split_idx = ((self.samples.len() as f64) * train_ratio) as usize;
        let (train_slice, test_slice) = self.samples.split_at(split_idx);

        (
            Dataset { samples: train_slice.to_vec() },
            Dataset { samples: test_slice.to_vec() },
        )
    }
}
