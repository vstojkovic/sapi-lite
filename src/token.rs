use std::ffi::OsString;
use std::mem::MaybeUninit;
use std::ptr::null_mut;

use windows as Windows;
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{
    IEnumSpObjectTokens, ISpObjectToken, ISpObjectTokenCategory, SpObjectTokenCategory,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::com_util::{from_wide, ComBox};
use crate::Result;

pub(crate) struct Token {
    pub(crate) intf: ISpObjectToken,
}

impl Token {
    pub fn attr(&self, name: &str) -> Result<OsString> {
        let attrs = unsafe { self.intf.OpenKey("Attributes") }?;
        let value = unsafe { ComBox::from_raw(attrs.GetStringValue(name)?) };
        Ok(unsafe { from_wide(&value) })
    }
}

pub(crate) struct Tokens {
    intf: IEnumSpObjectTokens,
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut intf = MaybeUninit::uninit();
        unsafe { self.intf.Next(1, intf.as_mut_ptr(), null_mut()) }.unwrap();
        unsafe { intf.assume_init() }.map(|intf| Token {
            intf,
        })
    }
}

pub(crate) struct Category {
    intf: ISpObjectTokenCategory,
}

impl Category {
    pub fn new(id: &str) -> Result<Self> {
        let intf: ISpObjectTokenCategory =
            unsafe { CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL) }?;
        unsafe { intf.SetId(id, false) }?;
        Ok(Self {
            intf,
        })
    }

    pub fn enum_tokens<S: AsRef<str>>(&self, req_attrs: S, opt_attrs: Option<S>) -> Result<Tokens> {
        use Windows::core::{IntoParam, Param};

        let opt_param: Param<PWSTR> = match opt_attrs {
            Some(attrs) => attrs.as_ref().into_param(),
            None => Param::None,
        };
        unsafe { self.intf.EnumTokens(req_attrs.as_ref(), opt_param.abi()) }.map(|intf| Tokens {
            intf,
        })
    }
}
