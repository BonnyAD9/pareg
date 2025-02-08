mod arg_into;
mod by_ref;
mod err;
mod from_arg;
mod from_read;
pub(crate) mod impl_all;
mod pareg_ref;
mod parsef;
mod parsers;
pub mod proc;
mod reader;
mod starts;

pub use crate::{
    arg_into::*, by_ref::*, err::*, from_arg::*, from_read::*, pareg_ref::*,
    parsef::*, parsers::*, reader::*,
};

use std::{borrow::Cow, cell::Cell, env, ops::Range};

/// Helper for parsing arguments.
///
/// The preffered way to use this is to call [`Pareg::ref_mut`] to get
/// [`ParegRef`] structure, which can be than used to parse the data:
///
/// This may be used to own the argument data. You can than get [`ParegRef`]
/// structure by calling [`Pareg::ref_mut`] to pass around and do the parsing,
/// because it can be less strict about lifetimes since it refers to the
/// original pareg structure and so it is more powerful.
pub struct Pareg {
    args: Vec<String>,
    cur: Cell<usize>,
}

impl From<Vec<String>> for Pareg {
    fn from(value: Vec<String>) -> Self {
        Self {
            args: value,
            cur: 0.into(),
        }
    }
}

impl Pareg {
    /// Create [`Pareg`] from vector of arguments. The first argument is NOT
    /// skipped.
    #[inline]
    pub fn new(args: Vec<String>) -> Self {
        args.into()
    }

    /// Create [`Pareg`] from [`env::args`], the first argument is skipped.
    #[inline]
    pub fn args() -> Self {
        Self {
            args: env::args().collect(),
            cur: 1.into(),
        }
    }

    /// DO NOT MAKE THIS PIBLIC. This can be public only if the lifetime
    /// captured inside [`ParegRef`] borrows the original [`Pareg`] mutably.
    #[inline(always)]
    pub(crate) fn inner(&self) -> ParegRef<'_, String> {
        ParegRef::new(&self.args, Cow::Borrowed(&self.cur))
    }

    /// Gets mutable reference to self. Mutating the resulting pareg ref will
    /// also mutate this pareg.
    #[inline]
    pub fn get_mut_ref(&mut self) -> ParegRef<'_, String> {
        // It is OK to pass the inner reference out, because this will borrow
        // [`Pareg`] mutably and so the captured reference in [`ParegRef`]
        // also borrows [`Pareg`] mutably.
        self.inner()
    }

    /// Gets immutable reference to self. Mutating the resulting pareg ref will
    /// not mutate this pareg.
    pub fn get_ref(&self) -> ParegRef<'_, String> {
        ParegRef::new(&self.args, Cow::Owned(self.cur.clone()))
    }

    /// Get the next argument
    // Iterator impl is not possible because the returned values are borrowed.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn next(&mut self) -> Option<&str> {
        self.inner().next()
    }

    /// Equivalent to calling next `cnt` times.
    #[inline]
    pub fn skip_args(&mut self, cnt: usize) -> Option<&str> {
        self.inner().skip_args(cnt)
    }

    /// Skip all remaining arguments and return the last.
    #[inline]
    pub fn skip_all(&mut self) -> Option<&str> {
        self.inner().skip_all()
    }

    /// Jump so that the argument at index `idx` is the next argument. Gets the
    /// argument at `idx - 1`.
    #[inline]
    pub fn jump(&mut self, idx: usize) -> Option<&str> {
        self.inner().jump(idx)
    }

    /// Jump to the zeroth argument.
    #[inline]
    pub fn reset(&mut self) {
        self.inner().reset()
    }

    /// Get the last returned argument.
    #[inline]
    pub fn cur(&self) -> Option<&str> {
        self.inner().cur()
    }

    /// Gets all the arguments (including the first one).
    #[inline]
    pub fn all_args(&self) -> &[String] {
        self.inner().all_args()
    }

    /// Gets the remaining arguments (not including the current).
    #[inline]
    pub fn remaining(&self) -> &[String] {
        self.inner().remaining()
    }

    /// Gets the remaining arguments (including the current).
    #[inline]
    pub fn cur_remaining(&self) -> &[String] {
        self.inner().cur_remaining()
    }

    /// Get value that will be returned with the next call to `next`.
    #[inline]
    pub fn peek(&self) -> Option<&str> {
        self.inner().peek()
    }

    /// Get the index of the next argument.
    #[inline]
    pub fn next_idx(&self) -> Option<usize> {
        self.inner().next_idx()
    }

    /// Get index of the current argument.
    #[inline]
    pub fn cur_idx(&self) -> Option<usize> {
        self.inner().cur_idx()
    }

    /// Get argument at the given index.
    #[inline]
    pub fn get(&self, idx: usize) -> Option<&str> {
        self.inner().get(idx)
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
    #[inline]
    pub fn next_manual<'a, T: 'a>(
        &'a mut self,
        f: impl Fn(&'a str) -> Result<T>,
    ) -> Result<T> {
        self.inner().next_manual(f)
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
    /// let res: (usize, f32) = args.cur_manual(|s| {
    ///     key_val_arg(s.strip_prefix("-D").unwrap(), '=')
    /// }).unwrap();
    /// assert_eq!((10, 0.25), res);
    /// ```
    #[inline]
    pub fn cur_manual<'a, T: 'a>(
        &'a self,
        f: impl Fn(&'a str) -> Result<T>,
    ) -> Result<T> {
        self.inner().cur_manual(f)
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
    pub fn next_arg<'a, T: FromArg<'a>>(&'a mut self) -> Result<T> {
        self.inner().next_arg()
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
    #[inline]
    pub fn next_key_mval<'a, K: FromArg<'a>, V: FromArg<'a>>(
        &'a mut self,
        sep: char,
    ) -> Result<(K, Option<V>)> {
        self.inner().next_key_mval(sep)
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
    #[inline]
    pub fn next_key_val<'a, K: FromArg<'a>, V: FromArg<'a>>(
        &'a mut self,
        sep: char,
    ) -> Result<(K, V)> {
        self.inner().next_key_val(sep)
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
    #[inline]
    pub fn next_bool(&mut self, t: &str, f: &str) -> Result<bool> {
        self.inner().next_bool(t, f)
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
    #[inline]
    pub fn next_opt_bool(
        &mut self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        self.inner().next_opt_bool(t, f, n)
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
    #[inline]
    pub fn next_key<'a, T: FromArg<'a>>(&'a mut self, sep: char) -> Result<T> {
        self.inner().next_key(sep)
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
    #[inline]
    pub fn next_val<'a, T: FromArg<'a>>(&'a mut self, sep: char) -> Result<T> {
        self.inner().next_val(sep)
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
    #[inline]
    pub fn next_mval<'a, T: FromArg<'a>>(
        &'a mut self,
        sep: char,
    ) -> Result<Option<T>> {
        self.inner().next_mval(sep)
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
    #[inline]
    pub fn cur_arg<'a, T: FromArg<'a>>(&'a self) -> Result<T> {
        self.inner().cur_arg()
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
    #[inline]
    pub fn cur_key_mval<'a, K: FromArg<'a>, V: FromArg<'a>>(
        &'a self,
        sep: char,
    ) -> Result<(K, Option<V>)> {
        self.inner().cur_key_mval(sep)
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
    #[inline]
    pub fn cur_key_val<'a, K: FromArg<'a>, V: FromArg<'a>>(
        &'a self,
        sep: char,
    ) -> Result<(K, V)> {
        self.inner().cur_key_val(sep)
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
    #[inline]
    pub fn cur_bool(&self, t: &str, f: &str) -> Result<bool> {
        self.inner().cur_bool(t, f)
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
    #[inline]
    pub fn cur_opt_bool(
        &self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>> {
        self.inner().cur_opt_bool(t, f, n)
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
    #[inline]
    pub fn cur_key<'a, T: FromArg<'a>>(&'a self, sep: char) -> Result<T> {
        self.inner().cur_key(sep)
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
    #[inline]
    pub fn cur_val<'a, T: FromArg<'a>>(&'a self, sep: char) -> Result<T> {
        self.inner().cur_val(sep)
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
    #[inline]
    pub fn cur_mval<'a, T: FromArg<'a>>(
        &'a self,
        sep: char,
    ) -> Result<Option<T>> {
        self.inner().cur_mval(sep)
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
    #[inline]
    pub fn cur_val_or_next<'a, T: FromArg<'a>>(
        &'a mut self,
        sep: char,
    ) -> Result<T> {
        self.inner().cur_val_or_next(sep)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur_with<'a, T>(
        &'a self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()> {
        self.inner().try_set_cur_with(res, f)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next_with<'a, T>(
        &'a mut self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()> {
        self.inner().try_set_next_with(res, f)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur<'a, T: FromArg<'a>>(
        &'a mut self,
        res: &mut Option<T>,
    ) -> Result<()> {
        self.inner().try_set_cur(res)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next<'a, T: FromArg<'a>>(
        &'a mut self,
        res: &mut Option<T>,
    ) -> Result<()> {
        self.inner().try_set_next(res)
    }

    /// Creates pretty error that the last argument (cur) is unknown.
    #[inline]
    pub fn err_unknown_argument(&self) -> ArgError {
        self.inner().err_unknown_argument()
    }

    /// Creates pretty error that there should be more arguments but there are
    /// no more arguments.
    #[inline]
    pub fn err_no_more_arguments(&self) -> ArgError {
        self.inner().err_no_more_arguments()
    }

    /// Creates error that says that the current argument has invalid value.
    #[inline]
    pub fn err_invalid(&self) -> ArgError {
        self.inner().err_invalid()
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_value(&self, value: String) -> ArgError {
        self.inner().err_invalid_value(value)
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_span(&self, span: Range<usize>) -> ArgError {
        self.inner().err_invalid_span(span)
    }

    /// Adds additional information to error so that it has better error
    /// message. Consider using [`ParegRef::cur_manual`] or
    /// [`ParegRef::next_manual`] instead.
    #[inline]
    pub fn map_err(&self, err: ArgError) -> ArgError {
        self.inner().map_err(err)
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
    /// let res: (usize, f32) = args.map_res(key_val_arg(arg, '=')).unwrap();
    /// assert_eq!((10, 0.25), res);
    /// ```
    #[inline]
    pub fn map_res<T>(&self, res: Result<T>) -> Result<T> {
        self.inner().map_res(res)
    }
}
