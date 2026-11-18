use std::time::Instant;
use rand::Rng;
use crate::crypto::{
    ExecutionMode, TargetAlgorithm, CryptoTrace,
    kyber::KyberHarness,
    dilithium::DilithiumHarness,
};
use crate::stats::TvlaEngine;
use crate::ml::{
    Dataset, LogisticRegression, DecisionForest, EvaluationResult, train_and_evaluate,
};

pub struct App {
    pub current_tab: usize,
    pub algo: TargetAlgorithm,
    pub mode: ExecutionMode,
    pub is_running: bool,
    pub traces: Vec<CryptoTrace>,
    pub tvla: TvlaEngine,
    pub sparkline_data: Vec<u64>,
    pub lr_eval: Option<EvaluationResult>,
    pub rf_eval: Option<EvaluationResult>,
    pub last_tick: Instant,
    pub status_msg: String,

    // Cryptographic harnesses
    kyber512: Option<KyberHarness>,
    kyber768: Option<KyberHarness>,
    dilithium: Option<DilithiumHarness>,

    sample_id: usize,
}

impl App {
    pub fn new() -> Self {
        let kyber512 = KyberHarness::new(TargetAlgorithm::Kyber512).ok();
        let kyber768 = KyberHarness::new(TargetAlgorithm::Kyber768).ok();
        let dilithium = DilithiumHarness::new().ok();

        Self {
            current_tab: 0,
            algo: TargetAlgorithm::Kyber512,
            mode: ExecutionMode::Hardened,
            is_running: true,
            traces: Vec::with_capacity(10_000),
            tvla: TvlaEngine::new(),
            sparkline_data: Vec::with_capacity(120),
            lr_eval: None,
            rf_eval: None,
            last_tick: Instant::now(),
            status_msg: "System initialized. Engine running live timing benchmark.".to_string(),
            kyber512,
            kyber768,
            dilithium,
            sample_id: 0,
        }
    }

    pub fn toggle_running(&mut self) {
        self.is_running = !self.is_running;
        self.status_msg = if self.is_running {
            "Benchmark RESUMED (Active sampling)".to_string()
        } else {
            "Benchmark PAUSED".to_string()
        };
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ExecutionMode::Hardened => ExecutionMode::Vulnerable,
            ExecutionMode::Vulnerable => ExecutionMode::Hardened,
        };
        self.status_msg = format!("Switched mode to {}", self.mode.as_str());
    }

    pub fn cycle_algorithm(&mut self) {
        self.algo = match self.algo {
            TargetAlgorithm::Kyber512 => TargetAlgorithm::Kyber768,
            TargetAlgorithm::Kyber768 => TargetAlgorithm::Dilithium2,
            TargetAlgorithm::Dilithium2 => TargetAlgorithm::Kyber512,
        };
        self.status_msg = format!("Target algorithm set to: {}", self.algo.name());
    }

    pub fn clear_data(&mut self) {
        self.traces.clear();
        self.tvla.clear();
        self.sparkline_data.clear();
        self.lr_eval = None;
        self.rf_eval = None;
        self.sample_id = 0;
        self.status_msg = "Cleared all traces and statistical buffers.".to_string();
    }

    pub fn train_models(&mut self) {
        if self.traces.len() < 30 {
            self.status_msg = "Need at least 30 samples to train ML distinguishers!".to_string();
            return;
        }

        let dataset = Dataset::from_traces(&self.traces);
        let lr = LogisticRegression::default();
        let rf = DecisionForest::default();

        self.lr_eval = Some(train_and_evaluate(lr, &dataset, 0.75));
        self.rf_eval = Some(train_and_evaluate(rf, &dataset, 0.75));

        self.status_msg = format!(
            "Trained ML Models on {} traces! LR AUC: {:.3} | RF AUC: {:.3}",
            self.traces.len(),
            self.lr_eval.as_ref().map(|e| e.roc_auc).unwrap_or(0.5),
            self.rf_eval.as_ref().map(|e| e.roc_auc).unwrap_or(0.5),
        );
    }

    pub fn export_csv(&mut self) {
        let file_path = "timing_traces.csv";
        let mut file = match std::fs::File::create(file_path) {
            Ok(f) => f,
            Err(e) => {
                self.status_msg = format!("Failed to create CSV: {}", e);
                return;
            }
        };

        use std::io::Write;
        let _ = writeln!(file, "id,algorithm,operation,mode,class,cycles,timestamp_ms");
        for t in &self.traces {
            let _ = writeln!(
                file,
                "{},{},{},{},{},{},{}",
                t.id,
                t.algo.name(),
                t.algo.operation_name(),
                t.mode.as_str(),
                t.class,
                t.cycles,
                t.timestamp_ms
            );
        }

        self.status_msg = format!("Exported {} traces to '{}' successfully!", self.traces.len(), file_path);
    }

    /// Ticks engine to generate a batch of trace samples
    pub fn tick(&mut self) {
        if !self.is_running {
            return;
        }

        let mut rng = rand::thread_rng();
        // Generate 6 samples per frame for smooth real-time visualization
        for _ in 0..6 {
            self.sample_id += 1;
            // Alternating / random 50% class distribution (TVLA Fixed vs Random)
            let class = if rng.gen_bool(0.5) { 0 } else { 1 };

            let cycles = match self.algo {
                TargetAlgorithm::Kyber512 => {
                    if let Some(harness) = &self.kyber512 {
                        harness.measure_decapsulation(self.mode, class)
                    } else {
                        self.mock_cycles(class)
                    }
                }
                TargetAlgorithm::Kyber768 => {
                    if let Some(harness) = &self.kyber768 {
                        harness.measure_decapsulation(self.mode, class)
                    } else {
                        self.mock_cycles(class)
                    }
                }
                TargetAlgorithm::Dilithium2 => {
                    if let Some(harness) = &self.dilithium {
                        harness.measure_signing(self.mode, class)
                    } else {
                        self.mock_cycles(class)
                    }
                }
            };

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);

            let trace = CryptoTrace {
                id: self.sample_id,
                algo: self.algo,
                mode: self.mode,
                class,
                cycles,
                timestamp_ms: now_ms,
            };

            self.tvla.push_sample(class, cycles);
            self.traces.push(trace);

            if self.sparkline_data.len() >= 100 {
                self.sparkline_data.remove(0);
            }
            self.sparkline_data.push(cycles);
        }
    }

    fn mock_cycles(&self, class: u8) -> u64 {
        let mut rng = rand::thread_rng();
        let base = 85_000u64 + rng.gen_range(0..250);
        if self.mode == ExecutionMode::Vulnerable && class == 1 {
            base + rng.gen_range(75..180)
        } else {
            base
        }
    }
}
