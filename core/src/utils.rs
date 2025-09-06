// Use this file for global utilities

use once_cell::sync;
use std::ffi::{
    CStr,
    c_char,
};

pub static LINUX_TYPE: sync::Lazy<String> = sync::Lazy::new(|| {
    std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "Not Linux".into())
});