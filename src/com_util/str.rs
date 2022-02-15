use std::ffi::OsString;
use std::os::windows::prelude::OsStringExt;

use windows as Windows;
use Windows::core::{IntoParam, Param};
use Windows::Win32::Foundation::PWSTR;

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
