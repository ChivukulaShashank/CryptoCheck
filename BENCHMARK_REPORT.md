### 1.1 Statistical Tests

| Metric | Value |
|--------|-------|
| `decaps_valid`   mean (cycles) | 117307.1 |
| `decaps_invalid` mean (cycles) | 119419.6 |
| `decaps_valid`   std            | 66955.8 |
| `decaps_invalid` std            | 70858.4 |
| Welch's t-statistic            | -1.0834 |
| Welch's t p-value              | 2.79e-01 |
| H₀ rejected (α=0.05)         | ❌ NO |
| KS statistic                   | 0.0232 |
| KS p-value                     | 5.16e-01 |
| KS H₀ rejected (α=0.05)      | ❌ NO |
| Cohen's d (effect size)        | -0.0306 |

> **Interpretation:** A rejected null hypothesis (p < α) indicates statistically
> significant timing differences. It does not by itself demonstrate key recovery or exploitability.

### 1.2 ML & Deep Learning Classifiers (Distinguishers)

| Classifier | Accuracy | AUC-ROC | 95% AUC CI | Precision | Recall | CV / Test AUC | Result |
|-----------|----------|---------|------------|-----------|--------|---------------|--------|
| Logistic Regression | 0.500 | 0.501 | 0.476–0.526 | 0.000 | 0.000 | 0.496 | Near/below baseline |
| Random Forest | 0.491 | 0.483 | 0.458–0.509 | 0.491 | 0.491 | 0.489 | Near/below baseline |
| Gradient Boosting | 0.458 | 0.455 | 0.429–0.480 | 0.467 | 0.596 | 0.494 | Near/below baseline |
| LSTM (PyTorch) | 0.500 | 0.472 | 0.418–0.524 | 0.500 | 1.000 | 0.472 | Near/below baseline |

### 1.3 Leakage Quantification

| Metric | Value |
|--------|-------|
| Mutual Information (bits) | 1.2695 |
| Shannon Entropy – valid (bits)   | 3.3132 |
| Shannon Entropy – invalid (bits) | 2.8022 |

---
## 4. ML-DSA-44 Sign/Verify Timing Analysis

Observed sign/verify timing variance:

| Phase | Mean (cycles) | Std Dev | CV (%) | Min | Max | Observations |
|-------|---------------|---------|--------|-----|-----|--------------|
| `ml_dsa_sign` | 351729.1 | 684006.6 | 194.47% | 103172 | 15113988 | 500 |
| `ml_dsa_verify` | 156992.5 | 58462.0 | 37.24% | 73436 | 518015 | 500 |

> **Finding:** Signing and verification show observed runtime variance. These timings
> warrant replication, but they do not by themselves prove rejection-sampling leakage.
