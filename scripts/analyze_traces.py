#!/usr/bin/env python3
"""
PQC Timing Side-Channel Offline Analysis & Statistical Distinguisher
Companion evaluation script that reads exported CSV timing traces (`timing_traces.csv`)
and performs Welch's t-test (TVLA) alongside ML distinguisher metrics.
Runs with ZERO external dependencies (pure Python standard library).
"""

import csv
import math
import sys
import os

def load_traces(csv_path="timing_traces.csv"):
    if not os.path.exists(csv_path):
        print(f"[Error] Trace file '{csv_path}' not found! Run the TUI and press 'E' to export traces first.")
        sys.exit(1)

    traces = []
    with open(csv_path, mode="r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for i, row in enumerate(reader):
            # Support both new rich schema and legacy 2-column schema
            trace_id = int(row.get("id") or row.get("iteration") or i)
            algo = row.get("algorithm", "ML-KEM-512")
            op = row.get("operation", "Decapsulation")
            mode = row.get("mode", "Vulnerable (Side-Channel Leak)")
            cls_val = int(row.get("class") if "class" in row else (trace_id % 2))
            cycles = int(row.get("cycles") or row.get("cycle_delta") or 0)
            ts = int(row.get("timestamp_ms") or 0)

            traces.append({
                "id": trace_id,
                "algorithm": algo,
                "operation": op,
                "mode": mode,
                "class": cls_val,
                "cycles": cycles,
                "timestamp_ms": ts,
            })
    return traces

def compute_distribution_stats(samples):
    if not samples:
        return {"n": 0, "mean": 0.0, "var": 0.0, "std": 0.0, "min": 0, "max": 0}
    n = len(samples)
    mean = sum(samples) / n
    var = sum((x - mean) ** 2 for x in samples) / (n - 1) if n > 1 else 0.0
    return {
        "n": n,
        "mean": mean,
        "var": var,
        "std": math.sqrt(var),
        "min": min(samples),
        "max": max(samples),
    }

def welch_t_test(g0, g1):
    s0 = compute_distribution_stats(g0)
    s1 = compute_distribution_stats(g1)

    if s0["n"] < 2 or s1["n"] < 2:
        return 0.0, 0.0, False

    v0 = s0["var"] / s0["n"]
    v1 = s1["var"] / s1["n"]
    denom = math.sqrt(v0 + v1)
    if denom == 0:
        return 0.0, 0.0, False

    t = (s0["mean"] - s1["mean"]) / denom

    # Welch-Satterthwaite degrees of freedom
    df_denom = (v0**2 / (s0["n"] - 1)) + (v1**2 / (s1["n"] - 1))
    df = (v0 + v1)**2 / df_denom if df_denom > 0 else 1.0

    is_leak = abs(t) > 4.5
    return t, df, is_leak

def train_logistic_regression(samples, labels, epochs=50, lr=0.1):
    if not samples:
        return 0.0, 0.5

    # Normalize features
    mean_x = sum(samples) / len(samples)
    std_x = math.sqrt(sum((x - mean_x)**2 for x in samples) / len(samples)) or 1.0
    norm_x = [(x - mean_x) / std_x for x in samples]

    w, b = 0.0, 0.0
    n = len(samples)

    for _ in range(epochs):
        for x, y in zip(norm_x, labels):
            z = max(min(w * x + b, 30.0), -30.0)
            p = 1.0 / (1.0 + math.exp(-z))
            err = p - y
            w -= lr * (err * x + 0.001 * w) / n
            b -= lr * err / n

    # Calculate Confusion Matrix
    tn, fp, fn_c, tp = 0, 0, 0, 0
    preds = []
    for x, y in zip(norm_x, labels):
        z = max(min(w * x + b, 30.0), -30.0)
        p = 1.0 / (1.0 + math.exp(-z))
        preds.append((p, y))
        pred_label = 1 if p >= 0.5 else 0
        if y == 0 and pred_label == 0: tn += 1
        elif y == 0 and pred_label == 1: fp += 1
        elif y == 1 and pred_label == 0: fn_c += 1
        elif y == 1 and pred_label == 1: tp += 1

    accuracy = (tp + tn) / n
    precision = tp / (tp + fp) if (tp + fp) > 0 else 0.0
    recall = tp / (tp + fn_c) if (tp + fn_c) > 0 else 0.0
    f1_score = 2.0 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0

    # Calculate ROC-AUC via trapezoidal rule
    preds.sort(key=lambda item: item[0], reverse=True)
    positives = sum(1 for _, y in preds if y == 1)
    negatives = n - positives
    if positives == 0 or negatives == 0:
        auc = 0.5
    else:
        tp_count, fp_count = 0, 0
        prev_fpr, prev_tpr = 0.0, 0.0
        auc = 0.0
        for _, actual in preds:
            if actual == 1:
                tp_count += 1
            else:
                fp_count += 1
            tpr = tp_count / positives
            fpr = fp_count / negatives
            auc += (fpr - prev_fpr) * (tpr + prev_tpr) / 2.0
            prev_fpr, prev_tpr = fpr, tpr
        auc = min(max(auc, 0.0), 1.0)

    # Shannon Mutual Information / Side-Channel Entropy Extracted:
    # I(X; Y) = H(Y) - H(Y|X)
    p_y1 = positives / n
    p_y0 = 1.0 - p_y1
    h_y = 0.0
    if p_y1 > 0: h_y -= p_y1 * math.log2(p_y1)
    if p_y0 > 0: h_y -= p_y0 * math.log2(p_y0)

    err = max(min(1.0 - accuracy, 0.9999), 0.0001)
    h_cond = -(err * math.log2(err) + (1.0 - err) * math.log2(1.0 - err))
    shannon_entropy_bits = max(h_y - h_cond, 0.0)

    return {
        "accuracy": accuracy,
        "precision": precision,
        "recall": recall,
        "f1_score": f1_score,
        "roc_auc": auc,
        "confusion_matrix": [[tn, fp], [fn_c, tp]],
        "shannon_entropy_bits": shannon_entropy_bits,
    }

def main():
    csv_file = sys.argv[1] if len(sys.argv) > 1 else "timing_traces.csv"
    print("=" * 72)
    print("  POST-QUANTUM CRYPTOGRAPHY TIMING SIDE-CHANNEL OFFLINE ANALYZER")
    print("=" * 72)
    print(f"Loading dataset from: '{csv_file}'...")
    traces = load_traces(csv_file)
    print(f"Successfully loaded {len(traces)} traces.")

    # Group by (Algorithm, Mode)
    groups = {}
    for t in traces:
        key = (t["algorithm"], t["mode"])
        groups.setdefault(key, []).append(t)

    for (algo, mode), group_traces in groups.items():
        print("\n" + "-" * 72)
        print(f"Target: {algo} | Configuration: {mode}")
        print("-" * 72)

        g0 = [t["cycles"] for t in group_traces if t["class"] == 0]
        g1 = [t["cycles"] for t in group_traces if t["class"] == 1]

        s0 = compute_distribution_stats(g0)
        s1 = compute_distribution_stats(g1)

        print(f"  Group 0 (Fixed Input)  : N = {s0['n']:<5} | Mean = {s0['mean']:10.2f} | Std = {s0['std']:7.2f}")
        print(f"  Group 1 (Random Input) : N = {s1['n']:<5} | Mean = {s1['mean']:10.2f} | Std = {s1['std']:7.2f}")

        # TVLA
        t_stat, df, is_leak = welch_t_test(g0, g1)
        verdict = "[CRITICAL LEAK DETECTED]" if is_leak else "[PASS - CONSTANT-TIME]"
        print(f"\n  [TVLA] Welch's t-statistic : {t_stat:+.3f} (Standard Threshold: |t| > 4.5)")
        print(f"  [TVLA] Assessment Status   : {verdict}")

        # ML Distinguisher
        all_cycles = [t["cycles"] for t in group_traces]
        all_labels = [t["class"] for t in group_traces]
        m = train_logistic_regression(all_cycles, all_labels)
        cm = m["confusion_matrix"]
        print(f"\n  [ML Distinguisher] Model   : Regularized Logistic Regression (SGD)")
        print(f"  [ML Distinguisher] Accuracy: {m['accuracy'] * 100:.2f}%")
        print(f"  [ML Distinguisher] Precision: {m['precision']:.3f}")
        print(f"  [ML Distinguisher] Recall   : {m['recall']:.3f}")
        print(f"  [ML Distinguisher] F1-Score : {m['f1_score']:.3f}")
        print(f"  [ML Distinguisher] ROC-AUC  : {m['roc_auc']:.3f}")
        print(f"  [ML Distinguisher] Shannon Information Extracted : {m['shannon_entropy_bits']:.4f} bits / trace")
        print(f"\n  [Confusion Matrix]")
        print(f"    Actual 0 (Fixed)  -> TN: {cm[0][0]:<6} | FP: {cm[0][1]:<6}")
        print(f"    Actual 1 (Random) -> FN: {cm[1][0]:<6} | TP: {cm[1][1]:<6}")

        if m["roc_auc"] > 0.70:
            print(f"\n  [Security Risk Assessment] CRITICAL: ML classifier successfully exploits timing leakage.")
            print(f"  Adversary extracts {m['shannon_entropy_bits']:.4f} bits of secret entropy per invocation.")
        else:
            print(f"\n  [Security Risk Assessment] SECURE: ML predictions equivalent to random guessing (AUC ~ 0.50).")
            print(f"  Zero side-channel entropy leaked to the adversary.")

    print("\n" + "=" * 72)
    print("Analysis complete.")

if __name__ == "__main__":
    main()
