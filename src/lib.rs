use std::ptr::null;

use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

mod com_util;
mod token;
pub mod tts;

pub type Error = windows::core::Error;
pub type Result<T> = windows::core::Result<T>;

pub fn initialize() -> Result<()> {
    unsafe { CoInitialize(null()) }
}

pub fn finalize() {
    unsafe { CoUninitialize() }
}
