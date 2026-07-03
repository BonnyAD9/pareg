use std::ffi::{OsStr, OsString};

use crate::{ArgInto, FromArg};

/// Type that will parse into a string even if the unicode is invalid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LossyString(pub String);

impl<'a> FromArg<'a> for LossyString {
    fn from_arg(arg: &'a str) -> crate::Result<Self> {
        Ok(arg.into())
    }

    fn from_os_arg(arg: &'a OsStr) -> crate::Result<Self> {
        Ok(arg.into())
    }
}

impl From<String> for LossyString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for LossyString {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<OsString> for LossyString {
    fn from(value: OsString) -> Self {
        Self(value.to_string_lossy().into_owned())
    }
}

impl From<&OsStr> for LossyString {
    fn from(value: &OsStr) -> Self {
        Self(value.to_string_lossy().into_owned())
    }
}

/// Convert the given string type into a string. If there are errors, recover.
pub fn arg_to_string_lossy<'a, S: ArgInto<'a>>(s: &'a S) -> String {
    let res: LossyString = s.arg_into().unwrap_or_default();
    res.0
}
