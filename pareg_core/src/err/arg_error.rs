use std::{borrow::Cow, ops::Range};

use thiserror::Error;

use super::{ArgErrCtx, ColorMode, Result};

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
    /// The value of argument was invalid.
    #[error("{0}")]
    InvalidValue(Box<ArgErrCtx>),
    /// Argument is specified too many times.
    #[error("{0}")]
    TooManyArguments(Box<ArgErrCtx>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
    /// Shortcut for creating parse error.
    pub fn parse_msg(msg: impl Into<Cow<'static, str>>, arg: String) -> Self {
        Self::FailedToParse(Box::new(ArgErrCtx::from_msg(msg, arg)))
    }

    /// Shortcut for creating parse error.
    pub fn value_msg(msg: impl Into<Cow<'static, str>>, arg: String) -> Self {
        Self::InvalidValue(Box::new(ArgErrCtx::from_msg(msg, arg)))
    }

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

    /// Set the span start.
    pub fn span_start(self, start: usize) -> Self {
        self.map_ctx(|c| c.span_start(start))
    }

    /// Sets the short message that is inlined with the code.
    pub fn inline_msg(self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.map_ctx(|c| c.inline_msg(msg))
    }

    /// Sets the primary (non inline) message.
    pub fn main_msg(self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.map_ctx(|c| c.main_msg(msg))
    }

    /// Set the color mode.
    pub fn color_mode(self, mode: ColorMode) -> Self {
        self.map_ctx(|c| c.color_mode(mode))
    }

    /// Disable color.
    pub fn no_color(self) -> Self {
        self.map_ctx(|c| c.no_color())
    }

    /// Sets new argument. If the original argument is substring of this,
    /// span will be adjusted.
    pub fn part_of(self, arg: String) -> Self {
        self.map_ctx(|c| c.part_of(arg))
    }

    pub fn postfix_of(self, arg: String) -> Self {
        self.map_ctx(|c| c.postfix_of(arg))
    }

    /// Helper method to wrap this in error and make it a result.
    pub fn err<T>(self) -> Result<T> {
        Err(self)
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
            ArgError::TooManyArguments(mut ctx) => {
                *ctx = f(*ctx);
                ArgError::TooManyArguments(ctx)
            }
            v => v,
        }
    }
}
