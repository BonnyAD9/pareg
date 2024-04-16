use std::borrow::Cow;

use thiserror::Error;

pub type Result<'a, T> = std::result::Result<T, ArgError<'a>>;

/// Errors thrown when parsing arguments.
#[derive(Debug, Error)]
pub enum ArgError<'a> {
    #[error("Unknown argument {0}.")]
    UnknownArgument(Cow<'a, str>),
    #[error(
        "Expected next argument{}.",
        if let Some(ref v) = .0 {
            format!(" after '{v}'")
        } else {
            "".to_owned()
        }
    )]
    NoMoreArguments(Option<Cow<'a, str>>),
    #[error(
        "Failed to parse '{value}' into '{typ}'{}",
        if let Some(msg) = .msg {
            format!(": {msg}")
        } else {
            ".".to_owned()
        }
    )]
    FailedToParse {
        typ: &'static str,
        value: Cow<'a, str>,
        msg: Option<Cow<'a, str>>,
    },
    #[error("Expected value, but there is no value in '{0}'.")]
    NoValue(Cow<'a, str>),
}

impl<'a> ArgError<'a> {
    pub fn into_owned(self) -> ArgError<'static> {
        match self {
            Self::UnknownArgument(a) => {
                ArgError::UnknownArgument(a.into_owned().into())
            }
            Self::FailedToParse { typ, value, msg } => {
                ArgError::FailedToParse {
                    typ,
                    value: value.into_owned().into(),
                    msg: msg.map(|a| a.into_owned().into()),
                }
            }
            Self::NoMoreArguments(a) => {
                ArgError::NoMoreArguments(a.map(|a| a.into_owned().into()))
            }
            Self::NoValue(a) => ArgError::NoValue(a.into_owned().into()),
        }
    }
}
