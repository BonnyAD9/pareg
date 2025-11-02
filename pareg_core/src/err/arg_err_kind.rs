use thiserror::Error;

/// Errors thrown when parsing arguments.
#[derive(Debug, Error)]
pub enum ArgErrKind {
    /// There was an unknown argument.
    #[error("Unknown argument.")]
    UnknownArgument,
    /// Expected another argument but there were no more arguments.
    #[error("No more arguments.")]
    NoMoreArguments,
    /// Failed to parse a string value into a type.
    #[error("Failed to parse.")]
    FailedToParse,
    /// There was no value in a key-value pair.
    #[error("No value.")]
    NoValue,
    /// The value of argument was invalid.
    #[error("Invalid value.")]
    InvalidValue,
    /// Argument is specified too many times.
    #[error("Too many arguments.")]
    TooManyArguments,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
