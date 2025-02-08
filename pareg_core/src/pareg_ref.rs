use std::{borrow::Cow, cell::Cell, ops::Range};

use crate::{
    bool_arg, key_arg, key_mval_arg, key_val_arg, mval_arg, opt_bool_arg,
    try_set_arg, try_set_arg_with, val_arg, ArgErrCtx, ArgError, ArgInto,
    ColorMode, FromArg, Result,
};

/// Helper for parsing arguments.
///
/// Reference to pareg structure. Mutating this can mutate original
/// [`crate::Pareg`] structure. You can use [`ParegRef::mutates_original`] to
/// check if this instance will mutate original pareg.
///
/// Note that the clones will not affect the original pareg even if the
/// original [`ParegRef`] did so you can clone if you don't want to mutate the
/// original.
///
/// In contrast to [`crate::Pareg`], it allows
/// calling mutable functions while there are immutable references to the
/// original arguments.
#[derive(Debug)]
pub struct ParegRef<'a, S: AsRef<str>> {
    args: &'a [S],
    cur: Cow<'a, Cell<usize>>,
}

impl<'a, S: AsRef<str>> ParegRef<'a, S> {
    /// Creates referenced pareg from arguments and current index.
    #[inline]
    pub fn new(args: &'a [S], cur: impl Into<Cow<'a, Cell<usize>>>) -> Self {
        Self {
            args,
            cur: cur.into(),
        }
    }

    /// Checks if this instance will mutate original [`crate::Pareg`]. If you
    /// don't want to mutate the orignal, you can create clone or call
    /// [`ParegRef::detach`].
    #[inline]
    pub fn mutates_original(&self) -> bool {
        matches!(self.cur, Cow::Borrowed(_))
    }

    /// Detach from the original pareg structure. Mutating this will no longer
    /// mutate the original [`crate::Pareg`] structure (if it did before).
    pub fn detach(&mut self) {
        self.cur = Cow::Owned(self.cur.as_ref().clone());
    }

    /// Get the last returned argument.
    #[inline]
    pub fn cur(&self) -> Option<&'a str> {
        let idx = self.cur.get();
        (idx != 0).then(|| self.args[idx - 1].as_ref())
    }

    /// Get argument at the given index.
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&'a str> {
        (idx < self.args.len()).then(|| self.args[idx].as_ref())
    }

    /// Get value that will be returned with the next call to `next`.
    #[inline]
    pub fn peek(&self) -> Option<&'a str> {
        self.get(self.cur.get())
    }

    /// Gets the remaining arguments (not including the current).
    #[inline]
    pub fn remaining(&self) -> &'a [S] {
        &self.args[self.cur.get()..]
    }

    /// Gets the remaining arguments (including the current).
    #[inline]
    pub fn cur_remaining(&self) -> &'a [S] {
        &self.args[self.cur.get().saturating_sub(1)..]
    }

    /// Gets all the arguments (including the first one).
    #[inline]
    pub fn all_args(&self) -> &'a [S] {
        self.args
    }

    /// Jump so that the argument at index `idx` is the next argument. Gets the
    /// argument at `idx - 1`.
    #[inline]
    pub fn jump(&mut self, idx: usize) -> Option<&'a str> {
        self.cur.set(idx.min(self.args.len()));
        self.cur()
    }

    /// Equivalent to calling next `cnt` times.
    #[inline]
    pub fn skip_args(&mut self, cnt: usize) -> Option<&'a str> {
        self.jump(self.cur.get() + cnt)
    }

    /// Skip all remaining arguments and return the last.
    #[inline]
    pub fn skip_all(&mut self) -> Option<&'a str> {
        self.jump(self.args.len())
    }

    /// Jump to the zeroth argument.
    #[inline]
    pub fn reset(&mut self) {
        self.jump(0);
    }

    /// Get the index of the next argument.
    #[inline]
    pub fn next_idx(&self) -> Option<usize> {
        let cur = self.cur.get();
        (cur < self.args.len()).then_some(cur)
    }

    /// Get index of the current argument.
    #[inline]
    pub fn cur_idx(&self) -> Option<usize> {
        let cur = self.cur.get();
        (cur != 0 && (cur - 1) < self.args.len()).then(|| cur - 1)
    }

    /// Perform manual parsing on the next argument. This is will make the
    /// errors have better messages than just doing the parsing without
    /// [`ParegRef`].
    ///
    /// `pareg.next_manual(foo)` is equivalent to
    /// `pareg.map_err(foo(pareg.next()))`.
    #[inline]
    pub fn next_manual<T>(
        &mut self,
        f: impl Fn(&'a str) -> Result<T>,
    ) -> Result<T> {
        let a = self.next_arg()?;
        self.map_res(f(a))
    }

    /// Perform manual parsing on the next argument. This is will make the
    /// errors have better messages than just doing the parsing without
    /// [`ParegRef`].
    ///
    /// `pareg.cur_manual(foo)` is equivalent to
    /// `pareg.map_err(foo(pareg.cur()))`.
    pub fn cur_manual<T>(
        &self,
        f: impl Fn(&'a str) -> Result<T>,
    ) -> Result<T> {
        self.map_res(f(self.cur_arg()?))
    }

    /// Parses the next value in the iterator.
    #[inline]
    pub fn next_arg<T: FromArg<'a>>(&mut self) -> Result<T> {
        if let Some(a) = self.next() {
            self.map_res(a.arg_into())
        } else {
            Err(self.err_no_more_arguments())
        }
    }

    /// Uses the function [`key_mval_arg`] on the next argument.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    #[inline]
    pub fn next_key_mval<K: FromArg<'a>, V: FromArg<'a>>(
        &mut self,
        sep: char,
    ) -> Result<(K, Option<V>)> {
        let arg = self.next_arg()?;
        self.map_res(key_mval_arg(arg, sep))
    }

    /// Uses the function [`key_val_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    #[inline]
    pub fn next_key_val<K: FromArg<'a>, V: FromArg<'a>>(
        &mut self,
        sep: char,
    ) -> Result<(K, V)> {
        let arg = self.next_arg()?;
        self.map_res(key_val_arg(arg, sep))
    }

    /// Uses the function [`bool_arg`] on the next value.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// otherwise returns error.
    #[inline]
    pub fn next_bool(&mut self, t: &str, f: &str) -> Result<bool> {
        let arg = self.next_arg()?;
        self.map_res(bool_arg(t, f, arg))
    }

    /// Uses the function [`opt_bool_arg`] on the next argument.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// if it is equal to `n` returns [`None`]. Otherwise returns error.
    #[inline]
    pub fn next_opt_bool(
        &mut self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        let arg = self.next_arg()?;
        self.map_res(opt_bool_arg(t, f, n, arg))
    }

    /// Uses the function [`key_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
    ///
    /// In case that there is no `'='`, parses the whole input.
    #[inline]
    pub fn next_key<T: FromArg<'a>>(&mut self, sep: char) -> Result<T> {
        let arg = self.next_arg()?;
        self.map_res(key_arg(arg, sep))
    }

    /// Uses the function [`val_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    #[inline]
    pub fn next_val<T: FromArg<'a>>(&mut self, sep: char) -> Result<T> {
        let arg = self.next_arg()?;
        self.map_res(val_arg(arg, sep))
    }

    /// Uses the function [`mval_arg`] on the next argument.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    #[inline]
    pub fn next_mval<T: FromArg<'a>>(
        &mut self,
        sep: char,
    ) -> Result<Option<T>> {
        let arg = self.next_arg()?;
        self.map_res(mval_arg(arg, sep))
    }

    /// Parses the last returned value from the iterator.
    #[inline]
    pub fn cur_arg<T: FromArg<'a>>(&self) -> Result<T> {
        if let Some(arg) = self.cur() {
            self.map_res(arg.arg_into())
        } else {
            Err(ArgError::NoLastArgument)
        }
    }

    /// Uses the function [`key_mval_arg`] on the last argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    #[inline]
    pub fn cur_key_mval<K: FromArg<'a>, V: FromArg<'a>>(
        &self,
        sep: char,
    ) -> Result<(K, Option<V>)> {
        self.map_res(key_mval_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`key_val_arg`] on the next value. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    #[inline]
    pub fn cur_key_val<K: FromArg<'a>, V: FromArg<'a>>(
        &self,
        sep: char,
    ) -> Result<(K, V)> {
        self.map_res(key_val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`bool_arg`] on the next value. If there is no last
    /// argument, returns `ArgError::NoLastArgument`.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// otherwise returns error.
    #[inline]
    pub fn cur_bool(&self, t: &str, f: &str) -> Result<bool> {
        self.map_res(bool_arg(t, f, self.cur_arg()?))
    }

    /// Uses the function [`opt_bool_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// if it is equal to `n` returns [`None`]. Otherwise returns error.
    #[inline]
    pub fn cur_opt_bool(
        &self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        self.map_res(opt_bool_arg(t, f, n, self.cur_arg()?))
    }

    /// Uses the function [`key_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
    ///
    /// In case that there is no `'='`, parses the whole input.
    #[inline]
    pub fn cur_key<T: FromArg<'a>>(&self, sep: char) -> Result<T> {
        self.map_res(key_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`val_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    #[inline]
    pub fn cur_val<T: FromArg<'a>>(&self, sep: char) -> Result<T> {
        self.map_res(val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`mval_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    #[inline]
    pub fn cur_mval<T: FromArg<'a>>(&self, sep: char) -> Result<Option<T>> {
        self.map_res(mval_arg(self.cur_arg()?, sep))
    }

    /// Split the current argument by the given separator and return the parsed
    /// value after the separator or if there is no such separator, parse the
    /// next argument.
    #[inline]
    pub fn cur_val_or_next<T: FromArg<'a>>(&mut self, sep: char) -> Result<T> {
        if let Some(res) = self.cur_mval(sep)? {
            Ok(res)
        } else {
            self.next_arg()
        }
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur_with<T>(
        &self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()> {
        self.map_res(try_set_arg_with(res, self.cur_arg()?, f))
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next_with<T>(
        &mut self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()> {
        let arg = self.next_arg()?;
        self.map_res(try_set_arg_with(res, arg, f))
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur<T: FromArg<'a>>(
        &self,
        res: &mut Option<T>,
    ) -> Result<()> {
        self.map_res(try_set_arg(res, self.cur_arg()?))
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next<T: FromArg<'a>>(
        &mut self,
        res: &mut Option<T>,
    ) -> Result<()> {
        let arg = self.next_arg()?;
        self.map_res(try_set_arg(res, arg))
    }

    /// Creates pretty error that the last argument (cur) is unknown.
    #[inline]
    pub fn err_unknown_argument(&self) -> ArgError {
        let arg = self.cur().unwrap_or_default();
        let long_message =
            self.cur().map(|a| format!("Unknown argument `{a}`").into());
        let ctx = ArgErrCtx {
            args: self.args.iter().map(|a| a.as_ref().to_string()).collect(),
            error_idx: self.cur.get().saturating_sub(1),
            error_span: 0..arg.len(),
            message: "Unknown argument.".into(),
            long_message,
            hint: None,
            color: Default::default(),
        };
        ArgError::UnknownArgument(ctx.into())
    }

    /// Creates error that says that the current argument has invalid value.
    #[inline]
    pub fn err_invalid(&self) -> ArgError {
        self.err_invalid_span(usize::MAX..usize::MAX)
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_value(&self, value: String) -> ArgError {
        self.map_err(ArgError::InvalidValue(Box::new(ArgErrCtx::from_msg(
            "Invalid value for argument.",
            value,
        ))))
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_span(&self, mut span: Range<usize>) -> ArgError {
        let value = self.cur().unwrap_or_default();
        if span.start > value.len() || span.end > value.len() {
            span = 0..value.len()
        }
        self.map_err(ArgError::InvalidValue(Box::new(ArgErrCtx::from_msg(
            "Invalid value for argument",
            String::new(),
        ))))
        .spanned(span)
    }

    /// Creates pretty error that there should be more arguments but there are
    /// no more arguments.
    pub fn err_no_more_arguments(&self) -> ArgError {
        let pos = self.args.last().map_or(0, |a| a.as_ref().len());
        let long_message = self.args.last().map(|a| {
            format!(
                "Expected more arguments after the argument `{}`.",
                a.as_ref()
            )
            .into()
        });
        let ctx = ArgErrCtx {
            args: self.args.iter().map(|a| a.as_ref().to_string()).collect(),
            error_idx: self.args.len().saturating_sub(1),
            error_span: pos..pos,
            message: "Expected more arguments.".into(),
            long_message,
            hint: None,
            color: ColorMode::default(),
        };
        ArgError::NoMoreArguments(ctx.into())
    }

    /// Adds additional information to error so that it has better error
    /// message. Consider using [`ParegRef::cur_manual`] or
    /// [`ParegRef::next_manual`] instead.
    #[inline(always)]
    pub fn map_err(&self, err: ArgError) -> ArgError {
        err.add_args(
            self.args.iter().map(|a| a.as_ref().to_string()).collect(),
            self.cur.get().saturating_sub(1),
        )
    }

    /// Adds additional information to error in result so that it has better
    /// error message. Consider using [`ParegRef::cur_manual`] or
    /// [`ParegRef::next_manual`] instead.
    #[inline(always)]
    pub fn map_res<T>(&self, res: Result<T>) -> Result<T> {
        res.map_err(|e| self.map_err(e))
    }
}

impl<'a, T: AsRef<str>> Iterator for ParegRef<'a, T> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur.get();
        (cur < self.args.len()).then(|| {
            self.cur.set(cur + 1);
            self.args[cur].as_ref()
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining().len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.remaining().len()
    }

    fn last(self) -> Option<Self::Item> {
        self.remaining().last().map(|s| s.as_ref())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let cur = self.cur.get();
        let new = cur + n;
        (new < self.args.len()).then(|| {
            self.cur.set(new);
            self.args[new].as_ref()
        })
    }
}

impl<T: AsRef<str>> Clone for ParegRef<'_, T> {
    /// Note that the clones will not affect the original pareg even if the
    /// original [`ParegRef`] did.
    fn clone(&self) -> Self {
        Self::new(self.args, Cow::Owned(self.cur.as_ref().clone()))
    }
}
