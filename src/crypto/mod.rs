pub mod timer;
pub mod kyber;
pub mod dilithium;

use serde::{Serialize, Deserialize};

/// Execution security configuration:
/// - `Hardened`: Constant-time baseline provided by standardized liboqs.
/// - `Vulnerable`: Emulates realistic secret-dependent timing leak (KyberSlash division or rejection sampling).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Hardened,
    Vulnerable,
}

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Hardened => "Hardened (Constant-Time)",
            ExecutionMode::Vulnerable => "Vulnerable (Side-Channel Leak)",
        }
    }
}

/// Supported NIST Post-Quantum Cryptographic Algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetAlgorithm {
    Kyber512,
    Kyber768,
    Dilithium2,
}

impl TargetAlgorithm {
    pub fn name(&self) -> &'static str {
        match self {
            TargetAlgorithm::Kyber512 => "ML-KEM-512 (Kyber-512)",
            TargetAlgorithm::Kyber768 => "ML-KEM-768 (Kyber-768)",
            TargetAlgorithm::Dilithium2 => "ML-DSA-44 (Dilithium-2)",
        }
    }

    pub fn operation_name(&self) -> &'static str {
        match self {
            TargetAlgorithm::Kyber512 | TargetAlgorithm::Kyber768 => "Decapsulation (kem.decapsulate)",
            TargetAlgorithm::Dilithium2 => "Signing (sig.sign)",
        }
    }
}

/// A single recorded cryptographic timing trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoTrace {
    pub id: usize,
    pub algo: TargetAlgorithm,
    pub mode: ExecutionMode,
    pub class: u8, // 0 = Fixed / Baseline, 1 = Random / Target
    pub cycles: u64,
    pub timestamp_ms: u128,
}
