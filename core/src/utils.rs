// Use this file for global utilities

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

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
