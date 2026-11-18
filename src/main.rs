use pqc_timing_harness::tui::{App, run_tui};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[System] Initializing PQC Timing Side-Channel Analysis Platform...");
    println!("[System] Initializing liboqs cryptographic engines (Kyber & Dilithium)...");

    // Initialize application state and cryptographic harnesses
    let mut app = App::new();

    // Launch interactive Terminal User Interface
    run_tui(&mut app)?;

    println!("[System] Platform exited cleanly. All traces and state safely managed.");
    Ok(())
}
