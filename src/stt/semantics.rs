use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::mem::ManuallyDrop;

use windows as Windows;
use Windows::core::{IntoParam, Param};
use Windows::Win32::Foundation::PWSTR;
use Windows::Win32::Media::Speech::{SPPHRASEPROPERTY, SPPROPERTYINFO};
use Windows::Win32::System::Com::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0};
use Windows::Win32::System::Ole::{VARENUM, VT_BOOL, VT_EMPTY, VT_I4, VT_R4, VT_R8};

use crate::com_util::from_wide;

pub trait SemanticString {
    fn as_os_str(&self) -> &OsStr;
}

impl<'s> SemanticString for &'s str {
    fn as_os_str(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl SemanticString for String {
    fn as_os_str(&self) -> &OsStr {
        OsStr::new(self)
    }
}

impl<'s> SemanticString for &'s OsStr {
    fn as_os_str(&self) -> &OsStr {
        self
    }
}

impl SemanticString for OsString {
    fn as_os_str(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl<'s> SemanticString for Cow<'s, str> {
    fn as_os_str(&self) -> &OsStr {
        match self {
            Cow::Borrowed(s) => s.as_os_str(),
            Cow::Owned(s) => s.as_os_str(),
        }
    }
}

impl<'s> SemanticString for Cow<'s, OsStr> {
    fn as_os_str(&self) -> &OsStr {
        match self {
            Cow::Borrowed(s) => *s,
            Cow::Owned(s) => s.as_os_str(),
        }
    }
}

#[derive(Debug)]
pub enum SemanticValue<S: SemanticString> {
    Bool(bool),
    Int(i32),
    Float(f32),
    Double(f64),
    String(S),
}

impl<S: SemanticString> SemanticValue<S> {
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
    fn from_sapi(property: &SPPHRASEPROPERTY) -> Result<Self, VARENUM> {
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

#[derive(Debug)]
pub struct SemanticTree {
    pub value: SemanticValue<OsString>,
    pub children: Vec<SemanticTree>,
}

impl SemanticTree {
    pub(crate) fn from_sapi(sapi_prop: Option<&SPPHRASEPROPERTY>) -> Vec<Self> {
        let mut result = Vec::new();
        let mut next_prop = sapi_prop;
        while let Some(prop) = next_prop {
            if let Ok(value) = SemanticValue::from_sapi(prop) {
                result.push(SemanticTree {
                    value,
                    children: SemanticTree::from_sapi(unsafe { prop.pFirstChild.as_ref() }),
                });
            }
            next_prop = unsafe { prop.pNextSibling.as_ref() };
        }
        result
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
