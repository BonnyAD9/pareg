use std::borrow::Cow;

use thiserror::Error;

pub type Result<'a, T> = std::result::Result<T, ArgError<'a>>;

/// Errors thrown when parsing arguments.
#[derive(Debug, Error)]
pub enum ArgError<'a> {
    /// There was an unknown argument. The string is the exact value of the
    /// argument that is not known.
    ///
    /// Prints the message: `Unknown argument '{0}'.`
    #[error("Unknown argument '{0}'.")]
    UnknownArgument(Cow<'a, str>),
    /// Expected another argument but there were no more arguments. The string
    /// is the last argument after which the next argument was expected.
    ///
    /// Prints the message:
    /// - `Expected next argument.` if the string is [`None`]
    /// - `Expected next argument after '{0}'` if the string is [`Some`]
    #[error(
        "Expected next argument{}.",
        if let Some(ref v) = .0 {
            format!(" after '{v}'")
        } else {
            "".to_owned()
        }
    )]
    NoMoreArguments(Option<Cow<'a, str>>),
    /// Failed to parse a string value into a type. `typ` is the name of the
    /// type, `value` is the string that failed to parse and `msg` may
    /// optionally contain more information.
    ///
    /// Prints the message:
    /// - `Failed to parse '{value}' into {typ}.` if `msg` is [`None`]
    /// - `Failed to parse '{value}' into {typ}: {msg}` if `msg` is [`Some`]
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
    /// There was no value in a key-value pair. The string is the value of the
    /// argument in which there was no value.
    ///
    /// Prints the message: `Expected value, but there is no value in '{0}'.`
    #[error("Expected value, but there is no value in '{0}'.")]
    NoValue(Cow<'a, str>),
}

impl<'a> ArgError<'a> {
    /// Converts the error into owned error by copying all borrowed strings.
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
