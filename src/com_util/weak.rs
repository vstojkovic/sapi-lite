use std::mem::{ManuallyDrop, transmute_copy};
use std::ops::{Deref, DerefMut};

use windows as Windows;
use Windows::core::{Interface, IntoParam, Param, IUnknown};

/// An incredibly dangerous wrapper around COM interface that should be used to avoid circular
/// references, and only when you're sure that the target will be valid for at least as long as
/// the wrapper lives.
pub struct MaybeWeak<I: Interface> {
    intf: ManuallyDrop<I>,
    is_weak: bool,
}

impl<I: Interface> MaybeWeak<I> {
    pub fn new(intf: I) -> Self {
        Self {
            intf: ManuallyDrop::new(intf),
            is_weak: false,
        }
    }

    pub fn is_weak(&self) -> bool {
        self.is_weak
    }

    pub fn set_weak(&mut self, weak: bool) {
        if self.is_weak != weak {
            self.is_weak = weak;
            if weak {
                self.release();
            } else {
                self.add_ref();
            }
        }
    }

    pub fn into_inner(mut self) -> I {
        if self.is_weak {
            self.add_ref();
        }
        self.is_weak = true;
        let intf = unsafe { ManuallyDrop::take(&mut self.intf) };
        intf
    }

    fn add_ref(&self) {
        let intf = &*self.intf;
        unsafe {
            Interface::assume_vtable::<IUnknown>(intf).1(transmute_copy(intf))
        };
    }

    fn release(&self) {
        let intf = &*self.intf;
        unsafe {
            Interface::assume_vtable::<IUnknown>(intf).2(transmute_copy(intf))
        };
    }
}

impl<I: Interface> Drop for MaybeWeak<I> {
    fn drop(&mut self) {
        if !self.is_weak {
            unsafe { ManuallyDrop::drop(&mut self.intf); }
        }
    }
}

impl<I: Interface> Deref for MaybeWeak<I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.intf
    }
}

impl<I: Interface> DerefMut for MaybeWeak<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.intf
    }
}

impl<'p, P: Interface, I: Interface + IntoParam<'p, P>> IntoParam<'p, P> for MaybeWeak<I> {
    fn into_param(self) -> Param<'p, P> {
        self.into_inner().into_param()
    }
}

impl<'p, P: Interface, I: Interface> IntoParam<'p, P> for &'p MaybeWeak<I>
where
    &'p I: IntoParam<'p, P>,
{
    fn into_param(self) -> Param<'p, P> {
        (&*self.intf).into_param()
    }
}
