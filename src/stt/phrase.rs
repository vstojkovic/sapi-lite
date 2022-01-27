use std::ffi::OsString;
use std::ptr::null_mut;

use windows as Windows;
use Windows::Win32::Media::Speech::{ISpRecoResult, SPPR_ALL_ELEMENTS};

use crate::com_util::{from_wide, out_to_ret, ComBox};
use crate::Result;

#[derive(Debug)]
pub struct Phrase {
    pub text: OsString,
}

impl Phrase {
    pub(crate) fn from_sapi(sapi_result: ISpRecoResult) -> Result<Self> {
        let text = unsafe {
            ComBox::from_raw(out_to_ret(|out| {
                sapi_result.GetText(
                    SPPR_ALL_ELEMENTS.0 as u32,
                    SPPR_ALL_ELEMENTS.0 as u32,
                    true,
                    out,
                    null_mut(),
                )
            })?)
        };
        Ok(Self {
            text: unsafe { from_wide(&text) },
        })
    }
}
