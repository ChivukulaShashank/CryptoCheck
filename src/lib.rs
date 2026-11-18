//! Post-Quantum Cryptography (PQC) Timing Side-Channel Analysis Engine
//!
//! Evaluates timing side-channel resistance of NIST PQC algorithms (Kyber / ML-KEM and Dilithium / ML-DSA)
//! through high-precision hardware cycle counting, statistical TVLA (Welch's t-test), and machine learning distinguishers.

pub mod crypto;
pub mod stats;
pub mod ml;
pub mod tui;

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::{ExecutionMode, TargetAlgorithm, CryptoTrace};
    use crypto::kyber::KyberHarness;
    use stats::TvlaEngine;
    use ml::{Dataset, LogisticRegression, train_and_evaluate};

    #[test]
    fn test_kyber_harness_decapsulation() {
        let harness = KyberHarness::new(TargetAlgorithm::Kyber512).expect("KEM init failed");
        let cycles_hardened = harness.measure_decapsulation(ExecutionMode::Hardened, 0);
        assert!(cycles_hardened > 0, "Cycle counter must be positive");

        let cycles_vulnerable = harness.measure_decapsulation(ExecutionMode::Vulnerable, 1);
        assert!(cycles_vulnerable > 0, "Cycle counter must be positive");
    }

    #[test]
    fn test_tvla_engine_welch_t_test() {
        let mut tvla = TvlaEngine::new();

        // Feed distinct distributions with variance to trigger leakage
        for i in 0..100 {
            tvla.push_sample(0, 100_000 + (i % 20)); // Fixed distribution
            tvla.push_sample(1, 100_500 + (i % 20)); // Shifted distribution
        }

        let verdict = tvla.evaluate_welch_t_test();
        assert!(verdict.is_leak(), "TVLA must detect significant distribution divergence");
    }

    #[test]
    fn test_ml_distinguisher_evaluation() {
        let mut traces = Vec::new();
        for i in 0..120 {
            let class = if i % 2 == 0 { 0 } else { 1 };
            let cycles = if class == 1 { 85_400 } else { 85_000 };
            traces.push(CryptoTrace {
                id: i,
                algo: TargetAlgorithm::Kyber512,
                mode: ExecutionMode::Vulnerable,
                class,
                cycles,
                timestamp_ms: 0,
            });
        }

        let dataset = Dataset::from_traces(&traces);
        let lr = LogisticRegression::default();
        let eval = train_and_evaluate(lr, &dataset, 0.75);

        assert!(eval.roc_auc > 0.50, "Trained model should achieve AUC > 0.50 on shifted traces");
    }
}

