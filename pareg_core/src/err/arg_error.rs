use std::{borrow::Cow, fmt::Display, ops::Range};

use crate::{ArgErrKind, ColorMode};

use super::{ArgErrCtx, Result};

/// Errors thrown when parsing arguments.
#[derive(Debug)]
pub struct ArgError(Box<ArgErrCtx>);

impl ArgError {
    pub fn new(ctx: ArgErrCtx) -> Self {
        Self(Box::new(ctx))
    }

    pub fn from_msg(
        kind: ArgErrKind,
        msg: impl Into<Cow<'static, str>>,
        arg: impl Into<String>,
    ) -> Self {
        Self(Box::new(ArgErrCtx::from_msg(kind, msg, arg.into())))
    }

    pub fn span_start(mut self, pos: usize) -> Self {
        self.0.span_start(pos);
        self
    }

    pub fn long_msg(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.0.long_msg(msg);
        self
    }

    pub fn err<T>(self) -> Result<T> {
        Err(self)
    }

    pub fn hint(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.0.hint(msg);
        self
    }

    pub fn shift_span(
        mut self,
        cnt: usize,
        new_arg: impl Into<String>,
    ) -> Self {
        self.0.shift_span(cnt, new_arg.into());
        self
    }

    pub fn spanned(mut self, span: Range<usize>) -> Self {
        self.0.spanned(span);
        self
    }

    pub fn inline_msg(mut self, msg: impl Into<Cow<'static, str>>) -> Self {
        self.0.inline_msg(msg);
        self
    }

    pub fn add_args(mut self, args: Vec<String>, idx: usize) -> Self {
        self.0.add_args(args, idx);
        self
    }

    pub fn part_of(mut self, arg: impl Into<String>) -> Self {
        self.0.part_of(arg.into());
        self
    }

    pub fn color_mode(mut self, mode: impl Into<ColorMode>) -> Self {
        self.0.color_mode(mode.into());
        self
    }

    pub fn no_color(mut self) -> Self {
        self.0.no_color();
        self
    }

    pub fn postfix_of(mut self, arg: impl Into<String>) -> Self {
        self.0.postfix_of(arg.into());
        self
    }

    pub fn anounce(mut self, anounce: bool) -> Self {
        self.0.anounce = anounce;
        self
    }

    pub fn invalid_value(
        msg: impl Into<Cow<'static, str>>,
        arg: impl Into<String>,
    ) -> Self {
        Self::from_msg(ArgErrKind::InvalidValue, msg, arg)
    }

    pub fn failed_to_parse(
        msg: impl Into<Cow<'static, str>>,
        arg: impl Into<String>,
    ) -> Self {
        Self::from_msg(ArgErrKind::FailedToParse, msg, arg)
    }

    pub fn too_many_arguments(
        msg: impl Into<Cow<'static, str>>,
        arg: impl Into<String>,
    ) -> Self {
        Self::from_msg(ArgErrKind::TooManyArguments, msg, arg)
    }

    pub fn kind(&self) -> &ArgErrKind {
        &self.0.kind
    }
    
    pub fn map_ctx(mut self, f: impl FnOnce(&mut ArgErrCtx)) -> Self {
        f(&mut self.0);
        self
    }
}

impl std::error::Error for ArgError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.kind.source()
    }
}

impl Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for ArgError
where
    T: Into<ArgErrKind>,
{
    fn from(value: T) -> Self {
        Self(Box::new(ArgErrCtx::new(value)))
    }
}

impl From<Box<ArgErrCtx>> for ArgError {
    fn from(value: Box<ArgErrCtx>) -> Self {
        Self(value)
    }
}
