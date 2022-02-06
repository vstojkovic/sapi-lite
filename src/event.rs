use windows as Windows;
use Windows::core::IUnknown;
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{
    ISpObjectToken, ISpRecoResult, SPEI_RECOGNITION, SPET_LPARAM_IS_OBJECT, SPET_LPARAM_IS_POINTER,
    SPET_LPARAM_IS_STRING, SPET_LPARAM_IS_TOKEN, SPET_LPARAM_IS_UNDEFINED, SPEVENT, SPEVENTENUM,
    SPEVENTLPARAMTYPE,
};

use crate::com_util::ComBox;
use crate::token::Token;
use crate::Result;

pub(crate) enum Event {
    Recognition(ISpRecoResult),
    OtherObject(IUnknown),
    OtherToken(Token),
    OtherString(ComBox<PWSTR>),
    OtherValue(ComBox<*const std::ffi::c_void>),
    Other,
}

impl Event {
    pub fn from_sapi(sapi_event: SPEVENT) -> Result<Self> {
        use Windows::core::{Abi, Interface};

        let id = SPEVENTENUM(sapi_event._bitfield & 0xffff);
        let lparam = sapi_event.lParam.0;
        match SPEVENTLPARAMTYPE(sapi_event._bitfield >> 16) {
            SPET_LPARAM_IS_OBJECT => {
                let intf = unsafe { IUnknown::from_abi(lparam as _) }?;
                match id {
                    SPEI_RECOGNITION => Ok(Self::Recognition(intf.cast()?)),
                    _ => Ok(Self::OtherObject(intf)),
                }
            }
            SPET_LPARAM_IS_POINTER => {
                Ok(Self::OtherValue(unsafe { ComBox::from_raw(lparam as _) }))
            }
            SPET_LPARAM_IS_STRING => {
                Ok(Self::OtherString(unsafe { ComBox::from_raw(PWSTR(lparam as _)) }))
            }
            SPET_LPARAM_IS_TOKEN => Ok(Self::OtherToken(Token::from_sapi(unsafe {
                ISpObjectToken::from_abi(lparam as _)
            }?))),
            SPET_LPARAM_IS_UNDEFINED => Ok(Self::Other),
            _ => panic!("Unrecognized SPEVENTLPARAMTYPE value"),
        }
    }
}
