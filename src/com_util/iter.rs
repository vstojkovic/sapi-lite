use std::mem::MaybeUninit;

use windows as Windows;
use Windows::core::Interface;

use crate::Result;

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
