use serde::{Serialize, Deserialize};

/// Statistical summary for a single sample distribution
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DistributionStats {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: u64,
    pub max: u64,
}

/// Leakage assessment status based on Welch's t-test threshold |t| > 4.5
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TvlaVerdict {
    InsufficientData,
    PassConstantTime(f64),
    FailLeakDetected(f64),
}

impl TvlaVerdict {
    pub fn is_leak(&self) -> bool {
        matches!(self, TvlaVerdict::FailLeakDetected(_))
    }

    pub fn t_score(&self) -> f64 {
        match self {
            TvlaVerdict::PassConstantTime(t) | TvlaVerdict::FailLeakDetected(t) => *t,
            TvlaVerdict::InsufficientData => 0.0,
        }
    }
}

/// Histogram bin for distribution visualization in TUI
#[derive(Debug, Clone, Default)]
pub struct HistogramBin {
    pub range_start: u64,
    pub range_end: u64,
    pub count_group0: usize,
    pub count_group1: usize,
}

/// Test Vector Leakage Assessment (TVLA) Engine using Welch's t-test
#[derive(Debug, Clone, Default)]
pub struct TvlaEngine {
    // Online Welford accumulator for Group 0 (Fixed)
    n0: usize,
    mean0: f64,
    m2_0: f64,
    min0: u64,
    max0: u64,
    samples0: Vec<u64>,

    // Online Welford accumulator for Group 1 (Random)
    n1: usize,
    mean1: f64,
    m2_1: f64,
    min1: u64,
    max1: u64,
    samples1: Vec<u64>,
}

impl TvlaEngine {
    pub fn new() -> Self {
        Self {
            min0: u64::MAX,
            min1: u64::MAX,
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Ingests a new trace measurement
    pub fn push_sample(&mut self, class: u8, cycles: u64) {
        if class == 0 {
            self.n0 += 1;
            let delta = cycles as f64 - self.mean0;
            self.mean0 += delta / self.n0 as f64;
            let delta2 = cycles as f64 - self.mean0;
            self.m2_0 += delta * delta2;

            self.min0 = self.min0.min(cycles);
            self.max0 = self.max0.max(cycles);
            self.samples0.push(cycles);
        } else {
            self.n1 += 1;
            let delta = cycles as f64 - self.mean1;
            self.mean1 += delta / self.n1 as f64;
            let delta2 = cycles as f64 - self.mean1;
            self.m2_1 += delta * delta2;

            self.min1 = self.min1.min(cycles);
            self.max1 = self.max1.max(cycles);
            self.samples1.push(cycles);
        }
    }

    pub fn stats_group0(&self) -> DistributionStats {
        let var = if self.n0 > 1 { self.m2_0 / (self.n0 - 1) as f64 } else { 0.0 };
        DistributionStats {
            count: self.n0,
            mean: self.mean0,
            variance: var,
            std_dev: var.sqrt(),
            min: if self.min0 == u64::MAX { 0 } else { self.min0 },
            max: self.max0,
        }
    }

    pub fn stats_group1(&self) -> DistributionStats {
        let var = if self.n1 > 1 { self.m2_1 / (self.n1 - 1) as f64 } else { 0.0 };
        DistributionStats {
            count: self.n1,
            mean: self.mean1,
            variance: var,
            std_dev: var.sqrt(),
            min: if self.min1 == u64::MAX { 0 } else { self.min1 },
            max: self.max1,
        }
    }

    /// Evaluates Welch's t-test between Group 0 and Group 1
    /// Formula: t = (x̄₀ - x̄₁) / sqrt(s₀²/N₀ + s₁²/N₁)
    pub fn evaluate_welch_t_test(&self) -> TvlaVerdict {
        if self.n0 < 15 || self.n1 < 15 {
            return TvlaVerdict::InsufficientData;
        }

        let s0 = self.stats_group0();
        let s1 = self.stats_group1();

        let v0 = s0.variance / self.n0 as f64;
        let v1 = s1.variance / self.n1 as f64;
        let denom = (v0 + v1).sqrt();

        if denom < 1e-12 {
            return TvlaVerdict::PassConstantTime(0.0);
        }

        let t = (s0.mean - s1.mean) / denom;

        // Classical cryptographic side-channel boundary: |t| > 4.5
        if t.abs() > 4.5 {
            TvlaVerdict::FailLeakDetected(t)
        } else {
            TvlaVerdict::PassConstantTime(t)
        }
    }

    /// Generates discrete distribution bins for histogram visualization
    pub fn compute_histogram_bins(&self, num_bins: usize) -> Vec<HistogramBin> {
        let s0 = self.stats_group0();
        let s1 = self.stats_group1();

        if s0.count == 0 && s1.count == 0 {
            return Vec::new();
        }

        let global_min = s0.min.min(s1.min);
        let global_max = s0.max.max(s1.max);

        if global_max <= global_min {
            return Vec::new();
        }

        let step = ((global_max - global_min) as f64 / num_bins as f64).max(1.0);
        let mut bins: Vec<HistogramBin> = (0..num_bins).map(|i| {
            let start = global_min + (i as f64 * step) as u64;
            let end = (start + step as u64).min(global_max);
            HistogramBin {
                range_start: start,
                range_end: end,
                count_group0: 0,
                count_group1: 0,
            }
        }).collect();

        // Bin samples for Group 0
        for &s in &self.samples0 {
            let idx = (((s - global_min) as f64 / step) as usize).min(num_bins - 1);
            bins[idx].count_group0 += 1;
        }

        // Bin samples for Group 1
        for &s in &self.samples1 {
            let idx = (((s - global_min) as f64 / step) as usize).min(num_bins - 1);
            bins[idx].count_group1 += 1;
        }

        bins
    }
}
