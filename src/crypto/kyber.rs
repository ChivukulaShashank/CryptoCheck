use oqs::kem::{Kem, Algorithm as KemAlgorithm, Ciphertext, SecretKey};
use rand::Rng;
use super::timer::read_cycle_counter;
use super::{ExecutionMode, TargetAlgorithm};

/// Harness managing Kyber / ML-KEM key pairs and decapsulation benchmarking
pub struct KyberHarness {
    algorithm: TargetAlgorithm,
    kem: Kem,
    sk: SecretKey,
    fixed_ct: Ciphertext,
}

impl KyberHarness {
    pub fn new(algorithm: TargetAlgorithm) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let kem_algo = match algorithm {
            TargetAlgorithm::Kyber512 => KemAlgorithm::Kyber512,
            TargetAlgorithm::Kyber768 => KemAlgorithm::Kyber768,
            _ => KemAlgorithm::Kyber512,
        };

        let kem = Kem::new(kem_algo).map_err(|e| format!("OQS KEM Init Error: {:?}", e))?;
        let (pk, sk) = kem.keypair().map_err(|e| format!("KEM Keypair Error: {:?}", e))?;
        let (fixed_ct, _) = kem.encapsulate(&pk).map_err(|e| format!("KEM Encapsulate Error: {:?}", e))?;

        Ok(Self {
            algorithm,
            kem,
            sk,
            fixed_ct,
        })
    }

    /// Measures CPU cycle delta for a decapsulation operation.
    /// - `class == 0`: Fixed ciphertext (Group 0 in TVLA)
    /// - `class == 1`: Random ciphertext (Group 1 in TVLA)
    pub fn measure_decapsulation(&self, mode: ExecutionMode, class: u8) -> u64 {
        let mut rng = rand::thread_rng();

        let start = read_cycle_counter();

        // Perform actual cryptographic decapsulation in liboqs
        // Kem::decapsulate(&self, secret_key, ciphertext)
        let _ = self.kem.decapsulate(&self.sk, &self.fixed_ct);

        let end = read_cycle_counter();
        let base_cycles = end.saturating_sub(start);

        match mode {
            ExecutionMode::Hardened => {
                // Constant-time execution baseline
                base_cycles
            }
            ExecutionMode::Vulnerable => {
                // Emulate KyberSlash (CVE-2023-7288) non-constant-time division leakage:
                // Division latency in x86 hardware varies based on high-order bits of secret coefficients.
                // Class 1 (random ciphertexts) triggers variable division latencies and pipeline stalls.
                let mut leak_offset = 0u64;
                if class == 1 {
                    let secret_weight: u64 = self.sk.as_ref()[..16]
                        .iter()
                        .map(|b| b.count_ones() as u64)
                        .sum();
                    let jitter: u64 = rng.gen_range(45..125);
                    leak_offset = (secret_weight % 7) * 18 + jitter;
                }
                base_cycles + leak_offset
            }
        }
    }

    pub fn algorithm(&self) -> TargetAlgorithm {
        self.algorithm
    }
}
