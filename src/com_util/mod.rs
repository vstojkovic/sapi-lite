use std::mem::MaybeUninit;

use crate::Result;

mod intf;
mod iter;
mod mem;
mod str;

pub use self::intf::Intf;
pub use self::iter::{next_elem, next_obj};
pub use self::mem::ComBox;
pub use self::str::{from_wide, opt_str_param};

pub unsafe fn out_to_ret<T, F: FnOnce(*mut T) -> Result<()>>(f: F) -> Result<T> {
    let mut result = MaybeUninit::uninit();
    f(result.as_mut_ptr())?;
    Ok(result.assume_init())
}
