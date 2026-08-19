//! Profiling integration with Tracy

#[cfg(feature = "tracy")]
use tracy_client::{span, Zone, plot, message};

/// Profiler interface
pub struct Profiler;

impl Profiler {
    /// Initialize profiler
    pub fn init() {
        #[cfg(feature = "tracy")]
        {
            // Tracy client auto-initializes on first use
        }
        tracing::info!("Profiler initialized");
    }

    /// Begin a profiling zone
    #[inline]
    pub fn begin_zone(name: &'static str) -> Option<ProfilerZone> {
        #[cfg(feature = "tracy")]
        {
            let zone = Zone::begin(name);
            Some(ProfilerZone { zone: Some(zone) })
        }
        #[cfg(not(feature = "tracy"))]
        {
            None
        }
    }

    /// Plot a value
    #[inline]
    pub fn plot(name: &'static str, value: f64) {
        #[cfg(feature = "tracy")]
        {
            plot(name, value);
        }
    }

    /// Send a message
    #[inline]
    pub fn message(text: &str) {
        #[cfg(feature = "tracy")]
        {
            message(text);
        }
    }
}

/// RAII profiling zone
pub struct ProfilerZone {
    #[cfg(feature = "tracy")]
    zone: Option<tracy_client::Zone<'static>>,
}

impl Drop for ProfilerZone {
    fn drop(&mut self) {
        #[cfg(feature = "tracy")]
        {
            if let Some(zone) = self.zone.take() {
                zone.end();
            }
        }
    }
}

/// Macro for profiling a function
#[macro_export]
macro_rules! profile_fn {
    () => {
        #[cfg(feature = "tracy")]
        let _zone = $crate::lithos_engine_profiler::Profiler::begin_zone(module_path!());
        #[cfg(not(feature = "tracy"))]
        let _zone: Option<$crate::lithos_engine_profiler::ProfilerZone> = None;
    };
}

/// Macro for profiling a scope
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "tracy")]
        let _zone = $crate::lithos_engine_profiler::Profiler::begin_zone($name);
        #[cfg(not(feature = "tracy"))]
        let _zone: Option<$crate::lithos_engine_profiler::ProfilerZone> = None;
    };
}

/// Macro for plotting a value
#[macro_export]
macro_rules! profile_plot {
    ($name:expr, $value:expr) => {
        $crate::lithos_engine_profiler::Profiler::plot($name, $value as f64);
    };
}

/// Macro for sending a message
#[macro_export]
macro_rules! profile_message {
    ($text:expr) => {
        $crate::lithos_engine_profiler::Profiler::message($text);
    };
}

/// Frame profiler for per-frame metrics
pub struct FrameProfiler {
    frame_number: u64,
    frame_start: std::time::Instant,
    zones: Vec<FrameZone>,
}

struct FrameZone {
    name: &'static str,
    start: std::time::Instant,
    end: Option<std::time::Instant>,
}

impl FrameProfiler {
    pub fn new() -> Self {
        Self {
            frame_number: 0,
            frame_start: std::time::Instant::now(),
            zones: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame_number += 1;
        self.frame_start = std::time::Instant::now();
        self.zones.clear();
    }

    pub fn end_frame(&mut self) {
        let frame_time = self.frame_start.elapsed();
        self.plot("frame.time_ms", frame_time.as_secs_f64() * 1000.0);
        self.plot("frame.number", self.frame_number as f64);
    }

    pub fn begin_zone(&mut self, name: &'static str) -> FrameZoneGuard {
        let zone = FrameZone { name, start: std::time::Instant::now(), end: None };
        let idx = self.zones.len();
        self.zones.push(zone);
        FrameZoneGuard { profiler: self, idx }
    }

    fn plot(&self, name: &'static str, value: f64) {
        #[cfg(feature = "tracy")]
        {
            tracy_client::plot(name, value);
        }
    }
}

/// Guard for frame zone
pub struct FrameZoneGuard<'a> {
    profiler: &'a mut FrameProfiler,
    idx: usize,
}

impl<'a> Drop for FrameZoneGuard<'a> {
    fn drop(&mut self) {
        self.profiler.zones[self.idx].end = Some(std::time::Instant::now());
        let zone = &self.profiler.zones[self.idx];
        let elapsed = zone.end.unwrap().duration_since(zone.start).as_secs_f64() * 1000.0;
        #[cfg(feature = "tracy")]
        {
            tracy_client::plot(zone.name, elapsed);
        }
    }
}

/// Global frame profiler
static mut GLOBAL_FRAME_PROFILER: Option<FrameProfiler> = None;

pub fn init_frame_profiler() {
    unsafe {
        GLOBAL_FRAME_PROFILER = Some(FrameProfiler::new());
    }
}

pub fn frame_profiler() -> &'static mut FrameProfiler {
    unsafe {
        GLOBAL_FRAME_PROFILER.as_mut().expect("Frame profiler not initialized")
    }
}