use std::ffi::OsString;

use windows as Windows;
use Windows::core::{IUnknown, IntoParam, Param};
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{
    IEnumSpObjectTokens, ISpObjectToken, ISpObjectTokenCategory, SpObjectToken,
    SpObjectTokenCategory,
};
use Windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

use crate::com_util::{from_wide, next_obj, opt_str_param, ComBox, Intf};
use crate::Result;

#[derive(Debug)]
pub(crate) struct Token {
    intf: Intf<ISpObjectToken>,
}

impl Token {
    pub fn new<'s, S: IntoParam<'s, PWSTR>>(id: S) -> Result<Self> {
        let intf: ISpObjectToken = unsafe { CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL) }?;
        unsafe { intf.SetId(None, id, false) }?;
        Ok(Token {
            intf: Intf(intf),
        })
    }

    pub fn from_sapi(intf: ISpObjectToken) -> Self {
        Token {
            intf: Intf(intf),
        }
    }

    pub fn to_sapi(self) -> ISpObjectToken {
        self.intf.0
    }

    pub fn attr(&self, name: &str) -> Result<OsString> {
        let attrs = unsafe { self.intf.OpenKey("Attributes") }?;
        let value = unsafe { ComBox::from_raw(attrs.GetStringValue(name)?) };
        Ok(unsafe { from_wide(&value) })
    }
}

impl<'p> IntoParam<'p, IUnknown> for Token {
    fn into_param(self) -> Param<'p, IUnknown> {
        self.intf.into_param()
    }
}

impl<'p> IntoParam<'p, ISpObjectToken> for Token {
    fn into_param(self) -> Param<'p, ISpObjectToken> {
        self.intf.into_param()
    }
}

pub(crate) struct Tokens {
    intf: Intf<IEnumSpObjectTokens>,
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { next_obj(&self.intf.0, IEnumSpObjectTokens::Next) }.ok()?.map(Token::from_sapi)
    }
}

pub(crate) struct Category {
    intf: Intf<ISpObjectTokenCategory>,
}

impl Category {
    pub fn new(id: &str) -> Result<Self> {
        let intf: ISpObjectTokenCategory =
            unsafe { CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL) }?;
        unsafe { intf.SetId(id, false) }?;
        Ok(Self {
            intf: Intf(intf),
        })
    }

    pub fn enum_tokens<S: AsRef<str>>(&self, req_attrs: S, opt_attrs: Option<S>) -> Result<Tokens> {
        unsafe { self.intf.EnumTokens(req_attrs.as_ref(), opt_str_param(opt_attrs).abi()) }.map(
            |intf| Tokens {
                intf: Intf(intf),
            },
        )
    }

    pub fn default_token(&self) -> Result<Token> {
        unsafe { self.intf.GetDefaultTokenId() }.and_then(Token::new)
    }
}
