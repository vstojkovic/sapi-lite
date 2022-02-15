use std::ffi::c_void;
use std::ops::Deref;

use windows as Windows;
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::System::Com::CoTaskMemFree;

pub unsafe trait ComBuffer {
    fn as_ptr(&self) -> *const c_void;
}

unsafe impl<T> ComBuffer for *const T {
    fn as_ptr(&self) -> *const c_void {
        *self as _
    }
}

unsafe impl ComBuffer for PWSTR {
    fn as_ptr(&self) -> *const c_void {
        self.0 as _
    }
}

#[derive(Debug)]
pub struct ComBox<P: ComBuffer>(P);

impl<P: ComBuffer> ComBox<P> {
    pub unsafe fn from_raw(ptr: P) -> Self {
        ComBox(ptr)
    }
}

impl<P: ComBuffer> Deref for ComBox<P> {
    type Target = P;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<P: ComBuffer> Drop for ComBox<P> {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(self.0.as_ptr()) }
    }
}
