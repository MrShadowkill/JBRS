use std::time::Instant;
pub struct Stats {
    start: Instant,
    pub frames: u32,
    pub fps: f32,
    pub draws: usize,
    pub rss_mb: f32,
    pub update_ms: f32,
    pub render_ms: f32,
}
impl Stats {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            frames: 0,
            fps: 0.,
            draws: 0,
            rss_mb: 0.,
            update_ms: 0.,
            render_ms: 0.,
        }
    }
    pub fn frame(&mut self) {
        self.frames += 1;
        if self.start.elapsed().as_secs_f32() >= 1. {
            self.fps = self.frames as f32 / self.start.elapsed().as_secs_f32();
            self.rss_mb = rss_mb();
            self.frames = 0;
            self.start = Instant::now();
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(windows))]
pub fn rss_mb() -> f32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<f32>().ok())
        })
        .unwrap_or(0.)
        / 1024.
}

#[cfg(windows)]
pub fn rss_mb() -> f32 {
    use std::ffi::c_void;
    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        faults: u32,
        peak_working_set: usize,
        working_set: usize,
        peak_paged: usize,
        paged: usize,
        peak_nonpaged: usize,
        nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }
    let mut counters: ProcessMemoryCounters = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;
    if unsafe {
        K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<ProcessMemoryCounters>() as u32,
        )
    } == 0
    {
        0.
    } else {
        counters.working_set as f32 / (1024. * 1024.)
    }
}
