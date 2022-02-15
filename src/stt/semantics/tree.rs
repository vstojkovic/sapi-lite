use std::ffi::OsString;

use windows as Windows;
use Windows::Win32::Media::Speech::SPPHRASEPROPERTY;

use super::SemanticValue;

#[derive(Debug, PartialEq, Clone)]
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
