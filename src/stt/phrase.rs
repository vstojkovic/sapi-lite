use std::ffi::OsString;
use std::ptr::null_mut;

use windows as Windows;
use Windows::Win32::Media::Speech::{ISpRecoResult, SPPHRASE_50, SPPR_ALL_ELEMENTS};

use crate::com_util::{from_wide, out_to_ret, ComBox};
use crate::Result;

use super::SemanticTree;

/// A successfully recognized phrase.
#[derive(Debug, PartialEq, Clone)]
pub struct Phrase {
    /// The text of the recognized phrase.
    pub text: OsString,
    /// The semantic information associated with the phrase.
    pub semantics: Vec<SemanticTree>,
}

impl Phrase {
    // Note: must be a recognized phrase, not a hypothesis or a false recognition
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
        let phrase_info =
            unsafe { ComBox::from_raw(sapi_result.GetPhrase()? as *const SPPHRASE_50) };
        let first_prop = unsafe { (*phrase_info).as_ref() }
            .and_then(|info| unsafe { info.pProperties.as_ref() });
        Ok(Self {
            text: unsafe { from_wide(&text) },
            semantics: SemanticTree::from_sapi(first_prop),
        })
    }
}
