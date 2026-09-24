//! leanVM `system-info` overlay with Windows support.
//!
//! Upstream at leanVM e2592df4 uses `libc::getrusage`, which is unavailable on
//! Windows. Peak RSS is best-effort there; thread and L1 helpers match upstream.

use std::sync::OnceLock;

const _: () = assert!(usize::BITS == 64, "this project requires a 64-bit target (for now)");

#[must_use]
pub fn num_threads() -> usize {
    static CACHE: OnceLock<usize> = OnceLock::new();
    *CACHE.get_or_init(|| {
        std::thread::available_parallelism()
            .expect("failed to detect available parallelism")
            .get()
    })
}

#[must_use]
pub fn l1_cache_size() -> usize {
    static CACHE: OnceLock<usize> = OnceLock::new();
    *CACHE.get_or_init(|| {
        detect_l1_cache_size().unwrap_or_else(|| {
            eprintln!("Warning: failed to detect L1 cache size, defaulting to 32 KB");
            32 * 1024
        })
    })
}

#[must_use]
pub fn peak_rss_bytes() -> u64 {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        peak_rss_unix()
    }
    #[cfg(windows)]
    {
        peak_rss_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        0
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn peak_rss_unix() -> u64 {
    let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &raw mut ru) };
    let max = ru.ru_maxrss as u64;
    // ru_maxrss unit: bytes on macOS, KiB on Linux.
    if cfg!(target_os = "macos") {
        max
    } else {
        max * 1024
    }
}

#[cfg(windows)]
fn peak_rss_windows() -> u64 {
    // PROCESS_MEMORY_COUNTERS.PeakWorkingSetSize (bytes).
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    extern "system" {
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn GetProcessMemoryInfo(
            process: *mut core::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }
    let mut pmc = unsafe { std::mem::zeroed::<ProcessMemoryCounters>() };
    pmc.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut pmc, pmc.cb) };
    if ok == 0 {
        0
    } else {
        pmc.peak_working_set_size as u64
    }
}

#[cfg(target_os = "linux")]
fn detect_l1_cache_size() -> Option<usize> {
    let s = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index0/size").ok()?;
    let s = s.trim();
    let last = s.chars().last()?;
    match last {
        'K' | 'k' => s[..s.len() - 1].parse::<usize>().ok().map(|n| n * 1024),
        'M' | 'm' => s[..s.len() - 1].parse::<usize>().ok().map(|n| n * 1024 * 1024),
        c if c.is_ascii_digit() => s.parse().ok(),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn detect_l1_cache_size() -> Option<usize> {
    let read_sysctl = |key: &str| -> Option<usize> {
        let out = std::process::Command::new("sysctl").args(["-n", key]).output().ok()?;
        std::str::from_utf8(&out.stdout).ok()?.trim().parse().ok()
    };
    read_sysctl("hw.perflevel0.l1dcachesize").or_else(|| read_sysctl("hw.l1dcachesize"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detect_l1_cache_size() -> Option<usize> {
    None
}
