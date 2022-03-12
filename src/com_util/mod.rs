use std::mem::MaybeUninit;

use crate::Result;

mod intf;
mod iter;
mod locale;
mod mem;
mod str;
mod weak;

pub use self::intf::Intf;
pub use self::iter::{next_elem, next_obj};
pub use self::locale::Locale;
pub use self::mem::ComBox;
pub use self::str::{from_wide, opt_str_param};
pub use self::weak::MaybeWeak;

pub unsafe fn out_to_ret<T, F: FnOnce(*mut T) -> Result<()>>(f: F) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    f(result.as_mut_ptr())?;
    Ok(result.assume_init())
}
