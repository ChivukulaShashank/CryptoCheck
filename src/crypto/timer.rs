//! High-resolution CPU cycle timer using hardware RDTSC with memory serialization fences.

/// Reads the x86_64 Time Stamp Counter (RDTSC) bounded by `_mm_lfence` serialization barriers.
/// This guarantees instruction execution cannot be reordered around the cycle read.
#[inline(always)]
pub fn read_cycle_counter() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_lfence();
        let cycles = core::arch::x86_64::_rdtsc();
        core::arch::x86_64::_mm_lfence();
        cycles
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        // Safe fallback for non-x86 platforms: nanosecond counter
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let start = START.get_or_init(std::time::Instant::now);
        start.elapsed().as_nanos() as u64
    }
}
