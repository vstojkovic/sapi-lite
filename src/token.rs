use std::ffi::OsString;

use windows as Windows;
use Windows::core::IntoParam;
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{
    IEnumSpObjectTokens, ISpObjectToken, ISpObjectTokenCategory, SpObjectToken,
    SpObjectTokenCategory,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::com_util::{from_wide, next_obj, opt_str_param, ComBox};
use crate::Result;

pub(crate) struct Token {
    pub(crate) intf: ISpObjectToken,
}

impl Token {
    pub fn new<'s, S: IntoParam<'s, PWSTR>>(id: S) -> Result<Self> {
        let intf: ISpObjectToken = unsafe { CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL) }?;
        unsafe { intf.SetId(None, id, false) }?;
        Ok(Token {
            intf,
        })
    }

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
        unsafe { next_obj(&self.intf, IEnumSpObjectTokens::Next) }.ok()?.map(|intf| Token {
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
        unsafe { self.intf.EnumTokens(req_attrs.as_ref(), opt_str_param(opt_attrs).abi()) }.map(
            |intf| Tokens {
                intf,
            },
        )
    }

    pub fn default_token(&self) -> Result<Token> {
        unsafe { self.intf.GetDefaultTokenId() }.and_then(Token::new)
    }
}
