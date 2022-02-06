use std::mem::MaybeUninit;
use std::ops::DerefMut;
use std::{
    ffi::{c_void, OsString},
    ops::Deref,
    os::windows::prelude::OsStringExt,
};

use windows as Windows;
use Windows::core::{Interface, IntoParam, Param};
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::System::Com::CoTaskMemFree;

use crate::Result;

pub unsafe fn from_wide(s: &PWSTR) -> OsString {
    let len = (0..).take_while(|&i| *s.0.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(s.0, len);
    OsString::from_wide(slice)
}

pub fn opt_str_param<'p, S: AsRef<str>>(opt: Option<S>) -> Param<'p, PWSTR> {
    match opt {
        Some(s) => s.as_ref().into_param(),
        None => Param::None,
    }
}

pub unsafe fn out_to_ret<T, F: FnOnce(*mut T) -> Result<()>>(f: F) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    f(result.as_mut_ptr())?;
    Ok(result.assume_init())
}

pub unsafe fn next_elem<I, R>(
    intf: &I,
    f: unsafe fn(&I, u32, *mut R, *mut u32) -> Result<()>,
) -> Result<Option<R>> {
    let mut result = MaybeUninit::uninit();
    let mut fetched = MaybeUninit::uninit();
    f(intf, 1, result.as_mut_ptr(), fetched.as_mut_ptr())?;
    Ok(if fetched.assume_init() > 0 {
        Some(result.assume_init())
    } else {
        None
    })
}

pub unsafe fn next_obj<I: Interface, R: Interface>(
    intf: &I,
    f: unsafe fn(&I, u32, *mut Option<R>, *mut u32) -> Result<()>,
) -> Result<Option<R>> {
    let mut result = MaybeUninit::uninit();
    let mut fetched = MaybeUninit::uninit();
    f(intf, 1, result.as_mut_ptr(), fetched.as_mut_ptr())?;
    Ok(if fetched.assume_init() > 0 {
        result.assume_init()
    } else {
        None
    })
}

// A zero-cost wrapper that makes a COM interface Send and Sync
pub struct Intf<I: Interface>(pub I);

unsafe impl<I: Interface> Send for Intf<I> {}
unsafe impl<I: Interface> Sync for Intf<I> {}

impl<I: Interface> Deref for Intf<I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: Interface> DerefMut for Intf<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'p, P: Interface, I: Interface + IntoParam<'p, P>> IntoParam<'p, P> for Intf<I> {
    fn into_param(self) -> Param<'p, P> {
        self.0.into_param()
    }
}

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
