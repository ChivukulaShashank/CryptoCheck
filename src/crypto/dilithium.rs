use oqs::sig::{Sig, Algorithm as SigAlgorithm, SecretKey};
use rand::{Rng, RngCore};
use super::timer::read_cycle_counter;
use super::{ExecutionMode, TargetAlgorithm};

/// Harness managing Dilithium / ML-DSA key pairs and signing benchmarking
pub struct DilithiumHarness {
    algorithm: TargetAlgorithm,
    sig: Sig,
    sk: SecretKey,
    fixed_msg: Vec<u8>,
}

impl DilithiumHarness {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let sig = Sig::new(SigAlgorithm::MlDsa44)
            .map_err(|e| format!("OQS SIG Init Error: {:?}", e))?;
        let (_pk, sk) = sig.keypair().map_err(|e| format!("SIG Keypair Error: {:?}", e))?;
        let fixed_msg = b"Standardized ML-DSA-44 Benchmark Vector".to_vec();

        Ok(Self {
            algorithm: TargetAlgorithm::Dilithium2,
            sig,
            sk,
            fixed_msg,
        })
    }

    /// Measures CPU cycle delta for a signature generation.
    /// - `class == 0`: Fixed message (Group 0 in TVLA)
    /// - `class == 1`: Random message (Group 1 in TVLA)
    pub fn measure_signing(&self, mode: ExecutionMode, class: u8) -> u64 {
        let mut rng = rand::thread_rng();

        let msg = if class == 0 {
            self.fixed_msg.clone()
        } else {
            let mut rand_msg = vec![0u8; 48];
            rng.fill_bytes(&mut rand_msg);
            rand_msg
        };

        let start = read_cycle_counter();

        // Perform actual cryptographic signing in liboqs
        let _ = self.sig.sign(&msg, &self.sk);

        let end = read_cycle_counter();
        let base_cycles = end.saturating_sub(start);

        match mode {
            ExecutionMode::Hardened => {
                // Constant-time execution baseline
                base_cycles
            }
            ExecutionMode::Vulnerable => {
                // Emulate rejection sampling non-constant-time behavior:
                // `rej_uniform` retries sample iterations when vector bounds are exceeded.
                // Class 1 messages with high Hamming weight have a realistic probability of triggering
                // an extra rejection sampling cycle.
                let mut leak_offset = 0u64;
                if class == 1 {
                    let reject_prob: f64 = rng.gen();
                    if reject_prob > 0.45 {
                        let sample_latency: u64 = rng.gen_range(120..350);
                        leak_offset = sample_latency;
                    }
                }
                base_cycles + leak_offset
            }
        }
    }

    pub fn algorithm(&self) -> TargetAlgorithm {
        self.algorithm
    }
}
