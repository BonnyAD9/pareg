use std::{borrow::Cow, ops::Range};

use thiserror::Error;

use super::ArgErrCtx;

/// Errors thrown when parsing arguments.
#[derive(Debug, Error)]
pub enum ArgError {
    /// There was an unknown argument.
    #[error("{0}")]
    UnknownArgument(Box<ArgErrCtx>),
    /// Expected another argument but there were no more arguments.
    #[error("{0}")]
    NoMoreArguments(Box<ArgErrCtx>),
    /// Failed to parse a string value into a type.
    #[error("{0}")]
    FailedToParse(Box<ArgErrCtx>),
    /// There was no value in a key-value pair.
    #[error("{0}")]
    NoValue(Box<ArgErrCtx>),
    #[error("{0}")]
    InvalidValue(Box<ArgErrCtx>),
    /// This error happens when you call any of the `cur_*` methods on
    /// [`crate::Pareg`]. It is not ment to happen in argument parsing and it
    /// may indicate that you have bug in your parsing.
    ///
    /// Prints the message: `"There was no last argument when it was expected.
    /// If you see this error, it is propably a bug."`
    #[error(
        "There was no last argument when it was expected. \
        If you see this error, it is propably a bug."
    )]
    NoLastArgument,
}

impl ArgError {
    /// Moves the span in the error message by `cnt` and changes the
    /// errornous argument to `new_arg`.
    pub fn shift_span(self, cnt: usize, new_arg: String) -> Self {
        self.map_ctx(|c| c.shift_span(cnt, new_arg))
    }

    /// Add arguments to the error so that it may have better error message.
    /// Mostly useful internaly in pareg.
    pub fn add_args(self, args: Vec<String>, idx: usize) -> Self {
        self.map_ctx(|c| c.add_args(args, idx))
    }

    /// Adds hint to the error message.
    pub fn hint(self, hint: impl Into<Cow<'static, str>>) -> Self {
        self.map_ctx(|c| c.hint(hint))
    }

    /// Adds span to the error message.
    pub fn spanned(self, span: Range<usize>) -> Self {
        self.map_ctx(|c| c.spanned(span))
    }

    pub fn map_ctx(self, f: impl FnOnce(ArgErrCtx) -> ArgErrCtx) -> Self {
        match self {
            ArgError::UnknownArgument(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::UnknownArgument(ctx)
            }
            ArgError::NoMoreArguments(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::NoMoreArguments(ctx)
            }
            ArgError::FailedToParse(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::FailedToParse(ctx)
            }
            ArgError::NoValue(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::NoValue(ctx)
            }
            ArgError::InvalidValue(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::InvalidValue(ctx)
            }
            ArgError::NoLastArgument => ArgError::NoLastArgument,
        }
    }
}
