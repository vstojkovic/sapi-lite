use std::ffi::{OsStr, OsString};
use std::mem::ManuallyDrop;

use windows as Windows;
use Windows::core::{IntoParam, Param};
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{SPPHRASEPROPERTY, SPPROPERTYINFO};
use Windows::Win32::System::Com::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0};
use Windows::Win32::System::Ole::{VARENUM, VT_BOOL, VT_EMPTY, VT_I4, VT_R4, VT_R8};

use crate::com_util::from_wide;

use super::SemanticString;

/// A value that forms part of the semantic information for a recognized phrase.
#[derive(Debug, PartialEq, Clone)]
#[allow(missing_docs)]
pub enum SemanticValue<S: SemanticString> {
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    String(S),
}

impl<S: SemanticString> SemanticValue<S> {
    /// Converts the value from one generic type into a compatible generic type.
    pub fn into<T: SemanticString>(self) -> SemanticValue<T>
    where
        S: Into<T>,
    {
        match self {
            Self::Bool(b) => SemanticValue::Bool(b),
            Self::Int(i) => SemanticValue::Int(i),
            Self::Float(f) => SemanticValue::Float(f),
            Self::Double(d) => SemanticValue::Double(d),
            Self::String(s) => SemanticValue::String(s.into()),
        }
    }

    /// Borrows the underlying value, if this is a `SemanticValue::Bool`.
    pub fn as_bool(&self) -> Option<&bool> {
        if let Self::Bool(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Borrows the underlying value, if this is a `SemanticValue::Int`.
    pub fn as_int(&self) -> Option<&i32> {
        if let Self::Int(i) = self {
            Some(i)
        } else {
            None
        }
    }

    /// Borrows the underlying value, if this is a `SemanticValue::Float`.
    pub fn as_float(&self) -> Option<&f32> {
        if let Self::Float(f) = self {
            Some(f)
        } else {
            None
        }
    }

    /// Borrows the underlying value, if this is a `SemanticValue::Double`.
    pub fn as_double(&self) -> Option<&f64> {
        if let Self::Double(d) = self {
            Some(d)
        } else {
            None
        }
    }

    /// Borrows the underlying value, if this is a `SemanticValue::String`.
    pub fn as_string(&self) -> Option<&S> {
        if let Self::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    fn to_pwstr<'s>(&self) -> Param<'s, PWSTR> {
        match self {
            SemanticValue::String(s) => s.as_os_str().into_param(),
            _ => Param::None,
        }
    }

    fn variant_type(&self) -> u16 {
        match self {
            SemanticValue::Bool(_) => VT_BOOL,
            SemanticValue::Int(_) => VT_I4,
            SemanticValue::Float(_) => VT_R4,
            SemanticValue::Double(_) => VT_R8,
            SemanticValue::String(_) => VT_EMPTY,
        }
        .0 as _
    }

    fn to_variant_union(&self) -> VARIANT_0_0_0 {
        match self {
            SemanticValue::Bool(b) => VARIANT_0_0_0 {
                boolVal: -(*b as i16),
            },
            SemanticValue::Int(i) => VARIANT_0_0_0 {
                lVal: *i,
            },
            SemanticValue::Float(f) => VARIANT_0_0_0 {
                fltVal: *f,
            },
            SemanticValue::Double(d) => VARIANT_0_0_0 {
                dblVal: *d,
            },
            SemanticValue::String(_) => Default::default(),
        }
    }
}

impl SemanticValue<OsString> {
    pub(super) fn from_sapi(property: &SPPHRASEPROPERTY) -> Result<Self, VARENUM> {
        if !property.pszValue.is_null() {
            Ok(Self::String(unsafe { from_wide(&property.pszValue) }.into()))
        } else {
            let var_type = unsafe { property.vValue.Anonymous.Anonymous.vt };
            let var_value = unsafe { &property.vValue.Anonymous.Anonymous.Anonymous };
            match VARENUM(var_type as _) {
                VT_BOOL => Ok(Self::Bool(unsafe { var_value.boolVal } != 0)),
                VT_I4 => Ok(Self::Int(unsafe { var_value.lVal })),
                VT_R4 => Ok(Self::Float(unsafe { var_value.fltVal })),
                VT_R8 => Ok(Self::Double(unsafe { var_value.dblVal })),
                vt @ _ => Err(vt),
            }
        }
    }
}

impl<S: SemanticString> From<bool> for SemanticValue<S> {
    fn from(source: bool) -> Self {
        Self::Bool(source)
    }
}

impl<S: SemanticString> From<i32> for SemanticValue<S> {
    fn from(source: i32) -> Self {
        Self::Int(source)
    }
}

impl<S: SemanticString> From<f32> for SemanticValue<S> {
    fn from(source: f32) -> Self {
        Self::Float(source)
    }
}

impl<S: SemanticString> From<f64> for SemanticValue<S> {
    fn from(source: f64) -> Self {
        Self::Double(source)
    }
}

impl<F: SemanticString + Into<T>, T: SemanticString> From<F> for SemanticValue<T> {
    fn from(source: F) -> Self {
        Self::String(source.into())
    }
}

impl<S: SemanticString> PartialEq<bool> for SemanticValue<S> {
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map(|value| value == other).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for bool {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_bool().map(|value| value == self).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<i32> for SemanticValue<S> {
    fn eq(&self, other: &i32) -> bool {
        self.as_int().map(|value| value == other).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for i32 {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_int().map(|value| value == self).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<f32> for SemanticValue<S> {
    fn eq(&self, other: &f32) -> bool {
        self.as_float().map(|value| value == other).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for f32 {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_float().map(|value| value == self).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<f64> for SemanticValue<S> {
    fn eq(&self, other: &f64) -> bool {
        self.as_double().map(|value| value == other).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for f64 {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_double().map(|value| value == self).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<&str> for SemanticValue<S> {
    fn eq(&self, other: &&str) -> bool {
        self.as_string().map(|value| value.as_os_str() == other.as_os_str()).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for &str {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_string().map(|value| value.as_os_str() == self.as_os_str()).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<&OsStr> for SemanticValue<S> {
    fn eq(&self, other: &&OsStr) -> bool {
        self.as_string().map(|value| value.as_os_str() == *other).unwrap_or(false)
    }
}

impl<S: SemanticString> PartialEq<SemanticValue<S>> for &OsStr {
    fn eq(&self, other: &SemanticValue<S>) -> bool {
        other.as_string().map(|value| value.as_os_str() == *self).unwrap_or(false)
    }
}

pub(crate) struct SemanticProperty<'s> {
    pub(crate) info: SPPROPERTYINFO,
    _pwstr: Param<'s, PWSTR>,
}

impl<'s> SemanticProperty<'s> {
    pub(crate) fn new<S: SemanticString>(value: &SemanticValue<S>) -> Self {
        let pwstr = value.to_pwstr();
        Self {
            info: SPPROPERTYINFO {
                pszValue: unsafe { pwstr.abi() },
                vValue: VARIANT {
                    Anonymous: VARIANT_0 {
                        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                            vt: value.variant_type(),
                            Anonymous: value.to_variant_union(),
                            ..Default::default()
                        }),
                    },
                },
                ..Default::default()
            },
            _pwstr: pwstr,
        }
    }
}
