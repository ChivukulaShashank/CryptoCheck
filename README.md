# CryptoCheck: TinyML & Post-Quantum Side-Channel Timing Harness

## 📖 What is This Project?
**CryptoCheck** is a lightweight research tool designed to measure and analyze side-channel timing variations in post-quantum cryptography (PQC) algorithms. 

When cryptographic software runs, different secret keys or inputs can cause microscopic differences in execution time. This project investigates whether those timing leaks can be captured and analyzed using everyday hardware.

### Why This Tech Stack?
* **Rust:** Used for the high-performance core engine. It handles low-level hardware timing (using CPU cycle counters) and interacts directly with cryptographic routines.
* **Lua:** Used as an orchestration and configuration layer. Instead of recompiling Rust code every time we want to change test settings, Lua scripts control parameters like sample sizes and iteration counts dynamically.
* **Everyday Hardware:** Built and run on standard consumer laptops (like an HP ProBook G5) to prove that advanced security analysis and machine learning can be performed locally without heavy cloud infrastructure.

---

## 🏗️ Project Architecture
* `src/main.rs`: The main execution runner that initializes the environment and collects timing data.
* `src/bin/lockpick.rs`: An auxiliary binary module used for targeted analysis or simulation loops.
* `config.lua` & `settings.lua`: Configuration files containing runtime rules, execution parameters, and test paths.
* `timing_traces.csv` & `lifecycle_timing_traces.csv`: Output datasets storing the captured raw cycle counts for subsequent analysis.

---

## ⚙️ Prerequisites
Before running the project, make sure your machine has:
1. **Rust toolchain** (Cargo and rustc)
2. **Lua runtime** (Lua 5.3 or higher)

---

## 🚀 Step-by-Step Run Instructions

Open your terminal and run the following commands sequentially:

### 1. Clone the Repository
```bash
git clone [https://github.com/ChivukulaShashank/CryptoCheck.git](https://github.com/ChivukulaShashank/CryptoCheck.git)
cd CryptoCheck
2. Check and Edit Configuration (Optional)
If you want to modify test parameters, open config.lua or settings.lua in any text editor:

Bash
nano config.lua
3. Build the Project
Compile the Rust binaries in optimized release mode:

Bash
cargo build --release
4. Execute the Timing Harness
Run the primary data collection and analysis engine:

Bash
cargo run --release
5. Run Auxiliary Analysis (Optional)
To run the alternate lockpicking/analysis module:

Bash
cargo run --release --bin lockpick
