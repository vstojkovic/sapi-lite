use std::ffi::OsString;
use std::str::FromStr;

use windows::Win32::Foundation::PWSTR;
use windows::Win32::Globalization::{LCIDToLocaleName, LocaleNameToLCID};
use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;

use super::from_wide;

pub struct Locale {
    lcid: u32,
}

impl Locale {
    pub fn new(lcid: u32) -> Self {
        Self { lcid }
    }

    pub fn lcid(&self) -> u32 {
        self.lcid
    }

    pub fn name(&self) -> OsString {
        let mut buffer: [u16; LOCALE_NAME_MAX_LENGTH as _] = [0; LOCALE_NAME_MAX_LENGTH as _];
        unsafe {
            LCIDToLocaleName(
                self.lcid,
                PWSTR(&mut buffer[0]),
                LOCALE_NAME_MAX_LENGTH as _,
                0,
            );
            from_wide(&PWSTR(&mut buffer[0]))
        }
    }
}

impl FromStr for Locale {
    type Err = windows::core::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lcid = unsafe { LocaleNameToLCID(s, 0) };
        if lcid != 0 {
            Ok(Self::new(lcid))
        } else {
            Err(Self::Err::from_win32())
        }
    }
}
