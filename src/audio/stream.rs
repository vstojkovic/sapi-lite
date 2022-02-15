use std::path::Path;
use std::ptr::null;

use windows as Windows;
use Windows::core::{GUID, HRESULT};
use Windows::Win32::Foundation::E_OUTOFMEMORY;
use Windows::Win32::Media::Speech::{
    ISpStream, SpStream, SPFILEMODE, SPFM_CREATE_ALWAYS, SPFM_OPEN_READONLY,
};
use Windows::Win32::System::Com::{CoCreateInstance, IStream, CLSCTX_ALL};
use Windows::Win32::UI::Shell::SHCreateMemStream;

use crate::com_util::Intf;
use crate::Result;

use super::AudioFormat;

pub struct AudioStream {
    intf: Intf<ISpStream>,
}

#[allow(non_upper_case_globals)]
const SPDFID_WaveFormatEx: GUID = GUID::from_u128(0xc31adbae_527f_4ff5_a230_f62bb61ff70c);

impl AudioStream {
    pub fn open_file<P: AsRef<Path>>(path: P, format: &AudioFormat) -> Result<Self> {
        Self::from_file(path, format, SPFM_OPEN_READONLY)
    }

    pub fn create_file<P: AsRef<Path>>(path: P, format: &AudioFormat) -> Result<Self> {
        Self::from_file(path, format, SPFM_CREATE_ALWAYS)
    }

    pub fn from_stream<S: Into<IStream>>(stream: S, format: &AudioFormat) -> Result<Self> {
        let intf: ISpStream = unsafe { CoCreateInstance(&SpStream, None, CLSCTX_ALL) }?;
        unsafe { intf.SetBaseStream(stream.into(), &SPDFID_WaveFormatEx, &format.to_sapi()) }?;
        Ok(Self {
            intf: Intf(intf),
        })
    }

    fn from_file<P: AsRef<Path>>(path: P, format: &AudioFormat, mode: SPFILEMODE) -> Result<Self> {
        let intf: ISpStream = unsafe { CoCreateInstance(&SpStream, None, CLSCTX_ALL) }?;
        unsafe {
            intf.BindToFile(
                path.as_ref().as_os_str(),
                mode,
                &SPDFID_WaveFormatEx,
                &format.to_sapi(),
                0,
            )
        }?;
        Ok(AudioStream {
            intf: Intf(intf),
        })
    }

    pub(crate) fn to_sapi(&self) -> ISpStream {
        self.intf.0.clone()
    }
}

pub struct MemoryStream {
    intf: Intf<IStream>,
}

impl MemoryStream {
    pub fn new(init_data: Option<&[u8]>) -> Result<Self> {
        Ok(Self {
            intf: Intf(Self::create_stream(init_data)?),
        })
    }

    pub fn try_clone(&self) -> Result<Self> {
        unsafe { self.intf.Clone() }.map(|intf| Self {
            intf: Intf(intf),
        })
    }

    fn create_stream(init_data: Option<&[u8]>) -> std::result::Result<IStream, HRESULT> {
        let size =
            init_data.map(|buf| buf.len()).unwrap_or(0).try_into().map_err(|_| E_OUTOFMEMORY)?;
        unsafe { SHCreateMemStream(init_data.map(|buf| buf.as_ptr()).unwrap_or(null()), size) }
            .ok_or(E_OUTOFMEMORY)
    }
}

impl From<MemoryStream> for IStream {
    fn from(source: MemoryStream) -> Self {
        source.intf.0
    }
}
