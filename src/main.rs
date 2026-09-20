use mlua::Lua;
use oqs::sig::{Sig, Algorithm};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[System] Initializing PQC Timing Data Collection Engine...");

    // 1. Load parameters from Lua
    let lua = Lua::new();
    let script_content = std::fs::read_to_string("script.lua")
        .unwrap_or_else(|_| "config = { iterations = 2000 }; return config.iterations".to_string());
    
    let iterations: u32 = lua.load(&script_content).eval()?;
    println!(" -> Loaded configuration from Lua: {} iterations per phase", iterations);

    // 2. Initialize liboqs (ML-DSA-44 / Dilithium)
    let sig = Sig::new(Algorithm::MlDsa44).map_err(|e| format!("OQS Error: {:?}", e))?;
    let message = b"HP ProBook G5 Full Lifecycle Security Test";

    // 3. Prepare CSV writer
    let mut csv_writer = File::create("lifecycle_timing_traces.csv")?;
    writeln!(csv_writer, "phase,iteration,cycle_delta")?;

    println!("[System] Capturing cycle traces across the Lock & Key lifecycle...");

    // --- PHASE 1: KEY GENERATION ---
    for i in 0..iterations {
        let start: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            start = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let _pair = sig.keypair().map_err(|e| format!("Keypair Error: {:?}", e))?;

        let end: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            end = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let delta = end - start;
        writeln!(csv_writer, "keypair,{},{}", i, delta)?;
    }

    let (pk, sk) = sig.keypair().map_err(|e| format!("Base Keypair Error: {:?}", e))?;
    let signature = sig.sign(message, &sk).map_err(|e| format!("Base Sign Error: {:?}", e))?;

    // --- PHASE 2: SIGNING ---
    for i in 0..iterations {
        let start: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            start = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let _sig = sig.sign(message, &sk).map_err(|e| format!("Sign Error: {:?}", e))?;

        let end: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            end = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let delta = end - start;
        writeln!(csv_writer, "sign,{},{}", i, delta)?;
    }

    // --- PHASE 3: VERIFICATION ---
    for i in 0..iterations {
        let start: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            start = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let _valid = sig.verify(message, &signature, &pk).is_ok();

        let end: u64;
        unsafe {
            std::arch::x86_64::_mm_lfence();
            end = std::arch::x86_64::_rdtsc();
            std::arch::x86_64::_mm_lfence();
        }

        let delta = end - start;
        writeln!(csv_writer, "verify,{},{}", i, delta)?;
    }

    println!("[System] Data collection complete. Traces saved to lifecycle_timing_traces.csv");
    Ok(())
}