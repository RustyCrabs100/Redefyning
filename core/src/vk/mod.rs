#![cfg(feature = "vulkan")]

#[path = "vulkan.rs"]
pub(crate) mod vulkan;

#[path = "vulkan.rs"]
pub(crate) use vulkan::*;
