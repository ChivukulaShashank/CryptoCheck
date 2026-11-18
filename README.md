# CryptoCheck: Post-Quantum Cryptography (PQC) Timing Side-Channel Analysis Platform

An interactive, modular research harness for detecting, measuring, and quantifying ML-exploitable timing side-channel leakage in NIST Post-Quantum Cryptography implementations (**ML-KEM / Kyber** and **ML-DSA / Dilithium**) using `liboqs`.

---

## 🔬 Core Objectives & Scientific Methodology

1. **Target Cryptographic Operations**:
   - **Kyber Decapsulation (`kem.decapsulate`)**: Evaluates **ML-KEM-512** and **ML-KEM-768**. Compares constant-time standard execution against secret-dependent polynomial division vulnerabilities (**KyberSlash**, CVE-2023-7288 / CVE-2023-7289).
   - **Dilithium Signature Generation (`sig.sign`)**: Evaluates **ML-DSA-44**. Models non-constant-time rejection sampling (`rej_uniform`) retry loop latency variations.
2. **High-Precision Cycle Telemetry**:
   - Employs hardware serialization instruction fences (`_mm_lfence`) around x86 Time Stamp Counter reads (`_rdtsc`) to guarantee pipeline serialization and cycle-accurate resolution.
3. **Statistical Leakage Assessment (TVLA)**:
   - Implements **Test Vector Leakage Assessment** via **Welch's $t$-test** between Group 0 (Fixed ciphertext baseline) and Group 1 (Random ciphertext target):
     $$t = \frac{\bar{X}_0 - \bar{X}_1}{\sqrt{\frac{s_0^2}{N_0} + \frac{s_1^2}{N_1}}}$$
   - Compares against standard cryptographic threshold: $|t| > 4.5 \implies$ **Fail (Leakage Detected)**, $|t| \le 4.5 \implies$ **Pass (Constant-Time Compliant)**.
4. **Machine Learning Distinguisher & Leakage Quantification**:
   - Trains and evaluates statistical distinguishers (**Regularized Logistic Regression** and **Random Forest Ensemble**).
   - Computes **Distinguisher Accuracy**, **ROC-AUC**, **Confusion Matrix**, and **Extracted Shannon Information (Entropy Bits)**.
   - **Negative Control Verification**:
     - *Hardened Configuration*: Distinguisher performs at random guessing ($\text{AUC} \approx 0.50$), proving side-channel immunity.
     - *Vulnerable Configuration*: Distinguisher successfully separates secret classes ($\text{AUC} > 0.80$, $\text{Accuracy} > 75\%$), confirming practical timing exploitability.

---

## 📁 Modular Codebase Architecture

```
pqc_timing_harness/
├── Cargo.toml                  # Project metadata & dependencies (oqs, ratatui, crossterm, serde)
├── README.md                   # Project documentation & architectural manual
├── scripts/
│   └── analyze_traces.py       # Offline Python TVLA & ML evaluation script (zero dependencies)
├── src/
│   ├── lib.rs                  # Module root & shared interfaces
│   ├── main.rs                 # CLI entry point & TUI runtime lifecycle manager
│   ├── bin/
│   │   └── lockpick.rs         # Auxiliary analysis binary
│   ├── crypto/                 # Cryptographic engines & hardware cycle timers
│   │   ├── mod.rs              # Execution modes (Hardened vs Vulnerable) & trace structures
│   │   ├── timer.rs            # RDTSC instruction fence timing
│   │   ├── kyber.rs            # ML-KEM-512 / 768 decapsulation harness
│   │   └── dilithium.rs        # ML-DSA-44 signing harness
│   ├── stats/                  # Statistical verification & TVLA engine
│   │   ├── mod.rs              # Re-exports
│   │   └── tvla.rs             # Welch's t-test, Welford's algorithm, distribution histograms
│   ├── ml/                     # Machine learning side-channel distinguishers
│   │   ├── mod.rs              # Training & evaluation pipeline
│   │   ├── dataset.rs          # Feature engineering, scaling, and train/test split
│   │   ├── models.rs           # Logistic Regression (SGD) & Random Forest Ensemble
│   │   └── metrics.rs          # Accuracy, Precision, Recall, ROC-AUC, Mutual Information
│   └── tui/                    # Terminal User Interface (Ratatui + Crossterm)
│       ├── mod.rs              # Event loop & terminal raw mode restoration
│       ├── app.rs              # Reactive application state & benchmark thread
│       └── ui.rs               # Multi-tab layout & interactive dashboard widgets
```

---

## 🚀 How to Run the Interactive TUI

### Quick Start
In your terminal (WSL or Linux with Rust installed):
```bash
cargo run --release
```
*(Or `cargo run` for debug mode)*

### Interactive Keyboard Controls

| Key | Action | Description |
| :---: | :---: | :--- |
| **`Tab`** / **`1` - `4`** | **Navigate Tabs** | Switch between Dashboard, Statistical TVLA, ML Distinguisher, and Settings |
| **`Space`** | **Pause / Resume** | Toggle real-time trace acquisition |
| **`M`** | **Toggle Security Mode** | Toggle between **`[HARDENED]`** (Constant-Time) and **`[VULNERABLE]`** (Timing Leak) |
| **`A`** | **Change Algorithm** | Cycle between **ML-KEM-512**, **ML-KEM-768**, and **ML-DSA-44** |
| **`T`** | **Train ML Distinguisher** | Train and evaluate classifiers on accumulated trace data |
| **`C`** | **Clear Traces** | Reset trace buffers and statistical accumulators |
| **`E`** | **Export CSV** | Export captured traces to `timing_traces.csv` |
| **`Q`** / **`Esc`** | **Quit** | Cleanly exit and restore terminal state |

---

## 📊 TUI Navigation Tabs

1. **Tab 1: Live Dashboard**: Real-time cycle sparkline chart, operational telemetry (Min, Max, Mean, Jitter), and recent execution trace logs.
2. **Tab 2: Statistical TVLA**: Welch's $t$-test evaluation, PASS/FAIL compliance badge, distribution statistics ($N, \mu, \sigma^2$), and dual-distribution ASCII histograms.
3. **Tab 3: ML Distinguishers**: Detailed confusion matrices, ROC-AUC comparisons, and comparative side-channel vulnerability assessments.
4. **Tab 4: Export & Settings**: Dataset persistence options, architectural hardware info, and command reference.
