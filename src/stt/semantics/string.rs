use std::borrow::Cow;
use std::ffi::{OsStr, OsString};

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
