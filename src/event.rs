use windows as Windows;
use Windows::core::{implement, IUnknown};
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{
    ISpEventSource, ISpNotifySink, ISpObjectToken, ISpRecoResult, SPEI_END_INPUT_STREAM,
    SPEI_RECOGNITION, SPEI_RESERVED1, SPEI_RESERVED2, SPET_LPARAM_IS_OBJECT,
    SPET_LPARAM_IS_POINTER, SPET_LPARAM_IS_STRING, SPET_LPARAM_IS_TOKEN, SPET_LPARAM_IS_UNDEFINED,
    SPEVENT, SPEVENTENUM, SPEVENTLPARAMTYPE,
};

use crate::com_util::{next_elem, ComBox, MaybeWeak};
use crate::token::Token;
use crate::Result;

#[derive(Debug)]
pub(crate) enum Event {
    Recognition(ISpRecoResult),
    SpeechFinished(u32),
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
            SPET_LPARAM_IS_STRING => Ok(Self::OtherString(unsafe {
                ComBox::from_raw(PWSTR(lparam as _))
            })),
            SPET_LPARAM_IS_TOKEN => Ok(Self::OtherToken(Token::from_sapi(unsafe {
                ISpObjectToken::from_abi(lparam as _)
            }?))),
            SPET_LPARAM_IS_UNDEFINED => match id {
                SPEI_END_INPUT_STREAM => Ok(Self::SpeechFinished(sapi_event.ulStreamNum)),
                _ => Ok(Self::Other),
            },
            _ => panic!("Unrecognized SPEVENTLPARAMTYPE value"),
        }
    }
}

pub(crate) struct EventSource {
    intf: MaybeWeak<ISpEventSource>,
}

impl EventSource {
    pub(crate) fn from_sapi(intf: ISpEventSource) -> Self {
        Self {
            intf: MaybeWeak::new(intf),
        }
    }

    pub(crate) fn next_event(&self) -> Result<Option<Event>> {
        Ok(
            match unsafe { next_elem(&*self.intf, ISpEventSource::GetEvents) }? {
                Some(sapi_event) => Some(Event::from_sapi(sapi_event)?),
                None => None,
            },
        )
    }

    fn downgrade(&mut self) {
        self.intf.set_weak(true);
    }
}

#[implement(Windows::Win32::Media::Speech::ISpNotifySink)]
pub(crate) struct EventSink {
    source: EventSource,
    handler: Box<dyn Fn(Event) -> Result<()>>,
}

#[allow(non_snake_case)]
impl EventSink {
    pub(crate) fn new<F: Fn(Event) -> Result<()> + 'static>(
        source: EventSource,
        handler: F,
    ) -> Self {
        Self {
            source,
            handler: Box::new(handler),
        }
    }

    pub(crate) fn install(self, interest: Option<&[SPEVENTENUM]>) -> Result<()> {
        use windows::core::ToImpl;

        let src_intf = self.source.intf.clone();
        let sink_intf: ISpNotifySink = self.into();
        unsafe { src_intf.SetNotifySink(&sink_intf) }?;
        unsafe { Self::to_impl(&sink_intf) }.source.downgrade();

        if let Some(flags) = interest {
            let mut flags_arg = (1u64 << SPEI_RESERVED1.0) | (1u64 << SPEI_RESERVED2.0);
            for flag in flags {
                flags_arg |= 1u64 << flag.0;
            }
            unsafe { src_intf.SetInterest(flags_arg, flags_arg) }?;
        }
        Ok(())
    }

    fn Notify(&self) -> Result<()> {
        while let Some(event) = self.source.next_event()? {
            (*self.handler)(event)?
        }
        Ok(())
    }
}
