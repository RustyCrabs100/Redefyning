// Use this file for global utilities

use {
    once_cell::sync::Lazy,
    raw_window_handle::{RawDisplayHandle, RawWindowHandle},
    std::time::{Duration, Instant},
};

pub(crate) static TIMER: Lazy<Instant> = Lazy::new(Instant::now);

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum AppState {
    Closed = 0,
    #[default]
    Awaiting = 1,
    Paused = 2,
    Loading = 4,
    Open = 8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RawWindowingHandles {
    window: RawWindowHandle,
    display: RawDisplayHandle,
}

impl RawWindowingHandles {
    pub(crate) fn from_raw_tuple(raw: &(RawDisplayHandle, RawWindowHandle)) -> Self {
        Self {
            window: raw.1,
            display: raw.0,
        }
    }

    pub(crate) fn unpack(self) -> (RawDisplayHandle, RawWindowHandle) {
        (self.display, self.window)
    }
}

unsafe impl Send for RawWindowingHandles {}
unsafe impl Sync for RawWindowingHandles {}

#[macro_export]
macro_rules! str_to_p_const_c_char {
    ($s:expr) => {{
        const BYTES: &[u8] = concat!($s, "\0").as_bytes();
        const REF_CSTR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(BYTES) };
        REF_CSTR.as_ptr()
    }};
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs_rem = secs % 60;
    let millis = d.subsec_millis();
    format!("{}:{:02}:{:02}.{:03}", hours, mins, secs_rem, millis)
}

#[macro_export]
macro_rules! logln {
    () => {{
        use std::io::Write;
        let elapsed = START.elapsed();
        let ts = format_duration(elapsed);
        let mut lock = io::stdout().lock();
        writeln!(lock, "Time Elapsed since Startup: {}", ts).expect("Write Failed");
        lock.flush().expect("Flush Failed");
    }};
    ($msg:expr, $location:expr) => {{
        use std::io::Write;
        let elapsed = START.elapsed();
        let ts = format_duration(elapsed);
        let mut lock = io::stdout().lock();
        let location_upper = format!("{}", $location).to_uppercase();
        let words = location_upper.split_whitespace()
            .map(|w| format!("[{}]", w.to_string()))
            .collect();
        let message = format!($fmt, $(, $args)*);
        let tags = words.join("");
        writeln!(lock, "Time Elapsed since Startup: {}", ts).expect("Write Failed");
        writeln!(lock, "[ENGINE]{}; {}", words, message);
        lock.flush().expect("Flush Failed");
    }};
    ($fmt:expr, $($args:expr)*, $location:expr) => {{
        use std::io::Write;
        let elapsed = START.elapsed();
        let ts = format_duration(elapsed);
        let mut lock = io::stdout().lock();
        let location_upper = format!("{}", $location).to_uppercase();
        let words = location_upper.split_whitespace()
            .map(|w| format!("[{}]", w.to_string()))
            .collect();
        let tags = words.join("");
        let message = format!($fmt, $(, $args)*);
        writeln!(lock, "Time Elapsed since Startup: {}", ts).expect("Write Failed");
        writeln!(lock, "[ENGINE]{}; {}", words, message);
        lock.flush().expect("Flush Failed");
    }};
}

/// Use this for debug mode ONLY, this will evaluate to nothing when debug_assertions is disabled & the debug feature.
#[macro_export]
macro_rules! debug_logln {
    () => {{
        #[cfg(all(debug_assertions, feature = "debug"))]
        {logln!()}
    }};
    ($msg:expr, $location:expr) => {{
        #[cfg(all(debug_assertions, feature = "debug"))]
        {logln!($msg, $location)}
    }};
    ($fmt:expr, $($args:expr)*, $location:expr) => {{
        #[cfg(all(debug_assertions, feature = "debug"))] {
            logln!($fmt, $($args)*, $location);
        }
    }};
}
