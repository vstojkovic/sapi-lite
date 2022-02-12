use std::ptr::null;

use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

pub mod audio;
mod com_util;
mod event;
pub mod stt;
mod token;
pub mod tts;

#[cfg(feature = "tokio")]
pub mod tokio;

pub type Error = windows::core::Error;
pub type Result<T> = windows::core::Result<T>;

pub fn initialize() -> Result<()> {
    unsafe { CoInitialize(null()) }
}

pub fn finalize() {
    unsafe { CoUninitialize() }
}
