use std::borrow::Cow;

use thiserror::Error;

/// Pareg result type. It is [`std::result::Result<T, ArgError<'a>>`]
pub type Result<T> = std::result::Result<T, ArgError>;

/// Errors thrown when parsing arguments.
#[derive(Debug, Error)]
pub enum ArgError {
    /// There was an unknown argument. The string is the exact value of the
    /// argument that is not known.
    ///
    /// Prints the message: `"Unknown argument '{0}'."`
    #[error("Unknown argument '{0}'.")]
    UnknownArgument(Cow<'static, str>),
    /// Expected another argument but there were no more arguments. The string
    /// is the last argument after which the next argument was expected.
    ///
    /// Prints the message:
    /// - `"Expected next argument."` if the string is [`None`]
    /// - `"Expected next argument after '{0}'"` if the string is [`Some`]
    #[error(
        "Expected next argument{}.",
        if let Some(ref v) = .0 {
            format!(" after '{v}'")
        } else {
            "".to_owned()
        }
    )]
    NoMoreArguments(Option<Cow<'static, str>>),
    /// Failed to parse a string value into a type. `typ` is the name of the
    /// type, `value` is the string that failed to parse and `msg` may
    /// optionally contain more information.
    ///
    /// Prints the message:
    /// - `Failed to parse '"{value}' into {typ}."` if `msg` is [`None`]
    /// - `Failed to parse '"{value}' into {typ}: {msg}"` if `msg` is [`Some`]
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
        value: Cow<'static, str>,
        msg: Option<Cow<'static, str>>,
    },
    /// There was no value in a key-value pair. The string is the value of the
    /// argument in which there was no value.
    ///
    /// Prints the message: `"Expected value, but there is no value in '{0}'."`
    #[error("Expected value, but there is no value in '{0}'.")]
    NoValue(Cow<'static, str>),
    /// This error happens when you call any of the `cur_*` methods on
    /// [`crate::ArgIterator`]. It is not ment to happen in argument parsing
    /// and it may indicate that you have bug in your parsing.
    ///
    /// Prints the message: `"There was no last argument when it was expected.
    /// If you see this error, it is propably a bug."`
    #[error(
        "There was no last argument when it was expected. \
        If you see this error, it is propably a bug."
    )]
    NoLastArgument,
}
