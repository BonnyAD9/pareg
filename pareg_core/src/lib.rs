mod arg_into;
mod by_ref;
mod err;
mod from_arg;
pub(crate) mod impl_all;
mod parsers;
pub mod proc;
mod starts;

pub use crate::{arg_into::*, by_ref::*, err::*, from_arg::*, parsers::*};

use std::{env, ops::Range};

/// Helper for parsing arguments.
pub struct Pareg {
    args: Vec<String>,
    cur: usize,
}

impl From<Vec<String>> for Pareg {
    fn from(value: Vec<String>) -> Self {
        Self {
            args: value,
            cur: 0,
        }
    }
}

impl Pareg {
    /// Create [`Pareg`] from vector of arguments. The first argument is NOT
    /// skipped.
    pub fn new(args: Vec<String>) -> Self {
        args.into()
    }

    /// Create [`Pareg`] from [`env::args`], the first argument is skipped.
    pub fn args() -> Self {
        Self {
            args: env::args().collect(),
            cur: 1,
        }
    }

    /// Get the next argument
    // Iterator impl is not possible because the returned values are borrowed.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&str> {
        next_inner(&self.args, &mut self.cur)
    }

    /// Get the last returned argument.
    pub fn cur(&self) -> Option<&str> {
        cur_inner(&self.args, self.cur)
    }

    /// Gets all the arguments (including the first one).
    pub fn all_args(&self) -> &[String] {
        &self.args
    }

    /// Gets the remaining arguments (not including the current).
    pub fn remaining(&self) -> &[String] {
        &self.args[self.args.len().min(self.cur + 1)..]
    }

    /// Gets the remaining arguments (including the current).
    pub fn cur_remaining(&self) -> &[String] {
        &self.args[self.cur..]
    }

    /// Perform manual parsing on the next argument. This is will make the
    /// errors have better messages than just doing the parsing without
    /// [`Pareg`].
    ///
    /// `pareg.next_manual(foo)` is equivalent to
    /// `pareg.map_err(foo(pareg.next()))`, except it has no issues with
    /// lifetimes.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::{Pareg, key_val_arg};
    /// let args = ["-D10=0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// let res: (usize, f32) = args.next_manual(|s| {
    ///     key_val_arg(s.strip_prefix("-D").unwrap(), '=')
    /// }).unwrap();
    /// assert_eq!((10, 0.25), res);
    /// ```
    pub fn next_manual<'a, T, F>(&'a mut self, f: F) -> Result<T>
    where
        T: 'a,
        F: Fn(&'a str) -> Result<T>,
    {
        self.next();
        self.map_err(f(self.cur_arg()?))
    }

    /// Perform manual parsing on the next argument. This is will make the
    /// errors have better messages than just doing the parsing without
    /// [`Pareg`].
    ///
    /// `pareg.cur_manual(foo)` is equivalent to
    /// `pareg.map_err(foo(pareg.cur()))`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::{Pareg, key_val_arg};
    /// let args = ["-D10=0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    ///
    /// let res: (usize, f32) = args.next_manual(|s| {
    ///     key_val_arg(s.strip_prefix("-D").unwrap(), '=')
    /// }).unwrap();
    /// assert_eq!((10, 0.25), res);
    /// ```
    pub fn cur_manual<'a, T, F>(&'a self, f: F) -> Result<T>
    where
        T: 'a,
        F: Fn(&'a str) -> Result<T>,
    {
        self.map_err(f(self.cur_arg()?))
    }

    /// Parses the next value in the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["hello", "10", "0.25", "always"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!("hello", args.next_arg::<&str>().unwrap());
    /// assert_eq!(10, args.next_arg::<usize>().unwrap());
    /// assert_eq!(0.25, args.next_arg::<f64>().unwrap());
    /// ```
    #[inline]
    pub fn next_arg<'a, T>(&'a mut self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        next_arg_inner(&self.args, &mut self.cur)
    }

    /// Uses the function [`key_mval_arg`] on the next argument.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     ("key", Some("value")),
    ///     args.next_key_mval::<&str, &str>('=').unwrap()
    /// );
    /// assert_eq!(
    ///     (5, Some(0.25)),
    ///     args.next_key_mval::<i32, f64>(':').unwrap()
    /// );
    /// assert_eq!(
    ///     ("only_key".to_owned(), None),
    ///     args.next_key_mval::<String, &str>('=').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_key_mval<'a, K, V>(
        &'a mut self,
        sep: char,
    ) -> Result<(K, Option<V>)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        self.next();
        self.map_err(key_mval_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`key_val_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     ("key", "value"),
    ///     args.next_key_val::<&str, &str>('=').unwrap()
    /// );
    /// assert_eq!(
    ///     (5, 0.25),
    ///     args.next_key_val::<i32, f64>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_key_val<'a, K, V>(&'a mut self, sep: char) -> Result<(K, V)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        self.next();
        self.map_err(key_val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`bool_arg`] on the next value.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["true", "yes", "never"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(true, args.next_bool("true", "false").unwrap());
    /// assert_eq!(true, args.next_bool("yes", "no").unwrap());
    /// assert_eq!(false, args.next_bool("always", "never").unwrap());
    /// ```
    #[inline(always)]
    pub fn next_bool(&mut self, t: &str, f: &str) -> Result<bool> {
        self.next();
        self.map_err(bool_arg(t, f, self.cur_arg()?))
    }

    /// Uses the function [`opt_bool_arg`] on the next argument.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// if it is equal to `n` returns [`None`]. Otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["always", "never", "auto"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     Some(true),
    ///     args.next_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// assert_eq!(
    ///     Some(false),
    ///     args.next_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// assert_eq!(
    ///     None,
    ///     args.next_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_opt_bool(
        &mut self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        self.next();
        self.map_err(opt_bool_arg(t, f, n, self.cur_arg()?))
    }

    /// Uses the function [`key_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
    ///
    /// In case that there is no `'='`, parses the whole input.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     "key",
    ///     args.next_key::<&str>('=').unwrap()
    /// );
    /// assert_eq!(
    ///     5,
    ///     args.next_key::<i32>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_key<'a, T>(&'a mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        self.next();
        self.map_err(key_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`val_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     "value",
    ///     args.next_val::<&str>('=').unwrap()
    /// );
    /// assert_eq!(
    ///     0.25,
    ///     args.next_val::<f64>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_val<'a, T>(&'a mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        self.next();
        self.map_err(val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`mval_arg`] on the next argument.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// assert_eq!(
    ///     Some("value"),
    ///     args.next_mval::<&str>('=').unwrap()
    /// );
    /// assert_eq!(
    ///     Some(0.25),
    ///     args.next_mval::<f64>(':').unwrap()
    /// );
    /// assert_eq!(
    ///     None,
    ///     args.next_mval::<&str>('=').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn next_mval<'a, T>(&'a mut self, sep: char) -> Result<Option<T>>
    where
        T: FromArg<'a>,
    {
        self.next();
        self.map_err(mval_arg(self.cur_arg()?, sep))
    }

    /// Parses the last returned value from the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["hello", "10", "0.25", "always"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!("hello", args.cur_arg::<&str>().unwrap());
    /// args.next();
    /// assert_eq!(10, args.cur_arg::<usize>().unwrap());
    /// args.next();
    /// assert_eq!(0.25, args.cur_arg::<f64>().unwrap());
    /// ```
    #[inline(always)]
    pub fn cur_arg<'a, T>(&'a self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        cur_arg_inner(&self.args, self.cur)
    }

    /// Uses the function [`key_mval_arg`] on the last argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     ("key", Some("value")),
    ///     args.cur_key_mval::<&str, &str>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     (5, Some(0.25)),
    ///     args.cur_key_mval::<i32, f64>(':').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     ("only_key".to_owned(), None),
    ///     args.cur_key_mval::<String, &str>('=').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_key_mval<'a, K, V>(
        &'a self,
        sep: char,
    ) -> Result<(K, Option<V>)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        self.map_err(key_mval_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`key_val_arg`] on the next value. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
    /// also parsed to the given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     ("key", "value"),
    ///     args.cur_key_val::<&str, &str>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     (5, 0.25),
    ///     args.cur_key_val::<i32, f64>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_key_val<'a, K, V>(&'a self, sep: char) -> Result<(K, V)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        self.map_err(key_val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`bool_arg`] on the next value. If there is no last
    /// argument, returns `ArgError::NoLastArgument`.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["true", "yes", "never"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(true, args.cur_bool("true", "false").unwrap());
    /// args.next();
    /// assert_eq!(true, args.cur_bool("yes", "no").unwrap());
    /// args.next();
    /// assert_eq!(false, args.cur_bool("always", "never").unwrap());
    /// ```
    #[inline(always)]
    pub fn cur_bool(&self, t: &str, f: &str) -> Result<bool> {
        self.map_err(bool_arg(t, f, self.cur_arg()?))
    }

    /// Uses the function [`opt_bool_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// if it is equal to `n` returns [`None`]. Otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["always", "never", "auto"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     Some(true),
    ///     args.cur_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     Some(false),
    ///     args.cur_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     None,
    ///     args.cur_opt_bool("always", "never", "auto").unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_opt_bool(
        &self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        self.map_err(opt_bool_arg(t, f, n, self.cur_arg()?))
    }

    /// Uses the function [`key_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
    ///
    /// In case that there is no `'='`, parses the whole input.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     "key",
    ///     args.cur_key::<&str>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     5,
    ///     args.cur_key::<i32>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_key<'a, T>(&'a self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        self.map_err(key_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`val_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, returns [`ArgError::NoValue`].
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     "value",
    ///     args.cur_val::<&str>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     0.25,
    ///     args.cur_val::<f64>(':').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_val<'a, T>(&'a self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        self.map_err(val_arg(self.cur_arg()?, sep))
    }

    /// Uses the function [`mval_arg`] on the next argument. If there is no
    /// last argument, returns `ArgError::NoLastArgument`.
    ///
    /// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
    /// given type.
    ///
    /// In case that there is no `'='`, value is `None`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     Some("value"),
    ///     args.cur_mval::<&str>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     Some(0.25),
    ///     args.cur_mval::<f64>(':').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     None,
    ///     args.cur_mval::<&str>('=').unwrap()
    /// );
    /// ```
    #[inline(always)]
    pub fn cur_mval<'a, T>(&'a self, sep: char) -> Result<Option<T>>
    where
        T: FromArg<'a>,
    {
        cur_mval_inner(&self.args, self.cur, sep)
    }

    /// Split the current argument by the given separator and return the parsed
    /// value after the separator or if there is no such separator, parse the
    /// next argument.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::Pareg;
    ///
    /// let args = ["--cnt", "20", "--cnt=10"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// assert_eq!(
    ///     20,
    ///     args.cur_val_or_next::<u32>('=').unwrap()
    /// );
    /// args.next();
    /// assert_eq!(
    ///     10,
    ///     args.cur_val_or_next::<u32>('=').unwrap()
    /// );
    /// ```
    pub fn cur_val_or_next<'a, T>(&'a mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        if let Some(res) = cur_mval_inner(&self.args, self.cur, sep)? {
            Ok(res)
        } else {
            next_arg_inner(&self.args, &mut self.cur)
        }
    }

    /// Creates pretty error that the last argument (cur) is unknown.
    pub fn err_unknown_argument(&self) -> ArgError {
        let arg = self.cur().unwrap_or("");
        let long_message = self
            .cur()
            .map(|a| format!("Unknown argument `{a}`.").into());
        let context = ArgErrCtx {
            args: self.args.clone(),
            error_idx: self.cur.saturating_sub(1),
            error_span: 0..arg.len(),
            message: "Unknown argument.".into(),
            long_message,
            hint: None,
            color: ColorMode::default(),
        };
        ArgError::UnknownArgument(context.into())
    }

    /// Creates pretty error that there should be more arguments but there are
    /// no more arguments.
    #[inline(always)]
    pub fn err_no_more_arguments(&self) -> ArgError {
        err_no_more_arguments_inner(&self.args)
    }

    /// Creates error that says that the current argument has invalid value.
    pub fn err_invalid(&self) -> ArgError {
        self.err_invalid_value(self.cur().unwrap_or_default().to_owned())
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    pub fn err_invalid_value(&self, value: String) -> ArgError {
        ArgError::InvalidValue(Box::new(ArgErrCtx::from_msg(
            "Invalid value for argument.",
            value,
        )))
        .add_args(self.args.clone(), self.cur.saturating_sub(1))
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    pub fn err_invalid_span(&self, span: Range<usize>) -> ArgError {
        let value = self.cur().unwrap_or_default();
        if span.start > value.len() || span.end > value.len() {
            self.err_invalid_value(value.to_owned())
        } else {
            ArgError::InvalidValue(Box::new(
                ArgErrCtx::from_msg(
                    "Invalid value for argument.",
                    value[span.clone()].to_owned(),
                )
                .spanned(span),
            ))
        }
    }

    /// Adds additional information to error so that it has better error
    /// message. Consider using [`Pareg::cur_manual`] or [`Pareg::next_manual`]
    /// instead.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::{Pareg, key_val_arg};
    /// let args = ["-D10=0.25"];
    /// let mut args = Pareg::new(args.iter().map(|a| a.to_string()).collect());
    ///
    /// args.next();
    /// let arg: &str = args.cur_arg().unwrap();
    /// let arg = arg.strip_prefix("-D").unwrap();
    ///
    /// let res: (usize, f32) = args.map_err(key_val_arg(arg, '=')).unwrap();
    /// assert_eq!((10, 0.25), res);
    /// ```
    pub fn map_err<T>(&self, res: Result<T>) -> Result<T> {
        map_err_inner(&self.args, self.cur, res)
    }
}

#[inline(always)]
fn cur_inner(args: &[String], cur: usize) -> Option<&str> {
    (cur != 0).then_some(&args[cur - 1])
}

#[inline(always)]
fn cur_arg_inner<'a, T>(args: &'a [String], cur: usize) -> Result<T>
where
    T: FromArg<'a>,
{
    if let Some(arg) = cur_inner(args, cur) {
        map_err_inner(args, cur, T::from_arg(arg))
    } else {
        Err(ArgError::NoLastArgument)
    }
}

#[inline(always)]
fn next_inner<'a>(args: &'a [String], cur: &mut usize) -> Option<&'a str> {
    (*cur < args.len()).then(|| {
        let res = &args[*cur];
        *cur += 1;
        res.as_str()
    })
}

#[inline(always)]
fn next_arg_inner<'a, T>(args: &'a [String], cur: &mut usize) -> Result<T>
where
    T: FromArg<'a>,
{
    if let Some(a) = next_inner(args, cur) {
        map_err_inner(args, *cur, a.arg_into())
    } else {
        Err(err_no_more_arguments_inner(args))
    }
}

#[inline(always)]
pub fn cur_mval_inner<'a, T>(
    args: &'a [String],
    cur: usize,
    sep: char,
) -> Result<Option<T>>
where
    T: FromArg<'a>,
{
    map_err_inner(args, cur, mval_arg(cur_arg_inner(args, cur)?, sep))
}

#[inline(always)]
fn map_err_inner<T>(args: &[String], cur: usize, res: Result<T>) -> Result<T> {
    res.map_err(|e| e.add_args(args.into(), cur.saturating_sub(1)))
}

pub fn err_no_more_arguments_inner(args: &[String]) -> ArgError {
    let pos = args.last().map_or(0, |a| a.len());
    let long_message = args.last().map(|a| {
        format!("Expected more arguments after the argument `{a}`.").into()
    });
    let context = ArgErrCtx {
        args: args.into(),
        error_idx: args.len() - 1,
        error_span: pos..pos,
        message: "Expected more arguments.".into(),
        long_message,
        hint: None,
        color: ColorMode::default(),
    };
    ArgError::NoMoreArguments(context.into())
}
