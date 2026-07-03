mod arg_into;
pub mod check;
mod err;
mod from_arg;
mod from_args;
pub(crate) mod impl_all;
mod lossy_string;
mod pareg_ref;
mod parsef;
mod parsers;
pub mod proc;
pub mod reader;
mod starts;
mod utils;

pub use crate::{
    arg_into::*,
    err::*,
    from_arg::*,
    from_args::*,
    lossy_string::*,
    pareg_ref::*,
    parsef::*,
    parsers::*,
    reader::{FromRead, ReadFmt, Reader, ReaderChars, SetFromRead},
};

use std::{borrow::Cow, cell::Cell, env, ffi::OsString, ops::RangeBounds};

/// Helper for parsing arguments.
///
/// The preffered way to use this is to call [`Pareg::ref_mut`] to get
/// [`ParegRef`] structure, which can be than used to parse the data:
///
/// This may be used to own the argument data. You can than get [`ParegRef`]
/// structure by calling [`Pareg::ref_mut`] to pass around and do the parsing,
/// because it can be less strict about lifetimes since it refers to the
/// original pareg structure and so it is more powerful.
pub struct Pareg<S = String> {
    args: Vec<S>,
    cur: Cell<usize>,
}

impl<S> From<Vec<S>> for Pareg<S> {
    fn from(value: Vec<S>) -> Self {
        Self {
            args: value,
            cur: 0.into(),
        }
    }
}

impl<S> Pareg<S> {
    /// Create [`Pareg`] from vector of arguments. The first argument is NOT
    /// skipped.
    #[inline]
    pub fn new(args: Vec<S>) -> Self {
        args.into()
    }

    /// DO NOT MAKE THIS PIBLIC. This can be public only if the lifetime
    /// captured inside [`ParegRef`] borrows the original [`Pareg`] mutably.
    #[inline(always)]
    pub(crate) fn inner<'a>(&'a self) -> ParegRef<'a, S>
    where
        S: ArgInto<'a>,
    {
        ParegRef::new(&self.args, Cow::Borrowed(&self.cur))
    }

    /// Gets mutable reference to self. Mutating the resulting pareg ref will
    /// also mutate this pareg.
    #[inline]
    pub fn get_mut_ref<'a>(&'a mut self) -> ParegRef<'a, S>
    where
        S: ArgInto<'a>,
    {
        // It is OK to pass the inner reference out, because this will borrow
        // [`Pareg`] mutably and so the captured reference in [`ParegRef`]
        // also borrows [`Pareg`] mutably.
        self.inner()
    }

    /// Gets immutable reference to self. Mutating the resulting pareg ref will
    /// not mutate this pareg.
    pub fn get_ref<'a>(&'a self) -> ParegRef<'a, S>
    where
        S: ArgInto<'a>,
    {
        ParegRef::new(&self.args, Cow::Owned(self.cur.clone()))
    }

    /// Get the next argument as string.
    ///
    /// Note that this may return Some("") if the conversion to string fails.
    /// (e.g. when converting invalid unicode from OsString).
    // Iterator impl is not possible because the returned values are borrowed.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn next_str<'a>(&'a mut self) -> Option<&'a str>
    where
        S: ArgInto<'a>,
    {
        self.inner().next_str()
    }

    /// Get the next argument
    // Iterator impl is not possible because the returned values are borrowed.
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn next<'a>(&'a mut self) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().next()
    }

    /// Equivalent to calling next `cnt` times.
    #[inline]
    pub fn skip_args<'a>(&'a mut self, cnt: usize) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().skip_args(cnt)
    }

    /// Skip all remaining arguments and return the last.
    #[inline]
    pub fn skip_all<'a>(&'a mut self) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().skip_all()
    }

    /// Jump so that the argument at index `idx` is the next argument. Gets the
    /// argument at `idx - 1`.
    #[inline]
    pub fn jump<'a>(&'a mut self, idx: usize) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().jump(idx)
    }

    /// Jump to the zeroth argument.
    #[inline]
    pub fn reset<'a>(&'a mut self)
    where
        S: ArgInto<'a>,
    {
        self.inner().reset()
    }

    /// Get the last returned argument.
    #[inline]
    pub fn cur<'a>(&'a self) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().cur()
    }

    /// Gets all the arguments (including the first one).
    #[inline]
    pub fn all_args<'a>(&'a self) -> &'a [S]
    where
        S: ArgInto<'a>,
    {
        self.inner().all_args()
    }

    /// Gets the remaining arguments (not including the current).
    #[inline]
    pub fn remaining<'a>(&'a self) -> &'a [S]
    where
        S: ArgInto<'a>,
    {
        self.inner().remaining()
    }

    /// Gets the remaining arguments (including the current).
    #[inline]
    pub fn cur_remaining<'a>(&'a self) -> &'a [S]
    where
        S: ArgInto<'a>,
    {
        self.inner().cur_remaining()
    }

    /// Get value that will be returned with the next call to `next`.
    #[inline]
    pub fn peek<'a>(&'a self) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
        self.inner().peek()
    }

    /// Get the index of the next argument.
    #[inline]
    pub fn next_idx<'a>(&'a self) -> Option<usize>
    where
        S: ArgInto<'a>,
    {
        self.inner().next_idx()
    }

    /// Get index of the current argument.
    #[inline]
    pub fn cur_idx<'a>(&'a self) -> Option<usize>
    where
        S: ArgInto<'a>,
    {
        self.inner().cur_idx()
    }

    /// Get argument at the given index.
    #[inline]
    pub fn get<'a>(&'a self, idx: usize) -> Option<&'a S>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    pub fn next_arg<'a, T: FromArg<'a>>(&'a mut self) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<(K, Option<V>)>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<(K, V)>
    where
        S: ArgInto<'a>,
    {
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
    pub fn next_bool<'a>(&'a mut self, t: &str, f: &str) -> Result<bool>
    where
        S: ArgInto<'a>,
    {
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
    pub fn next_opt_bool<'a>(
        &'a mut self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>>
    where
        S: ArgInto<'a>,
    {
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
    pub fn next_key<'a, T: FromArg<'a>>(&'a mut self, sep: char) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    pub fn next_val<'a, T: FromArg<'a>>(&'a mut self, sep: char) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<Option<T>>
    where
        S: ArgInto<'a>,
    {
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
    pub fn cur_arg<'a, T: FromArg<'a>>(&'a self) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<(K, Option<V>)>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<(K, V)>
    where
        S: ArgInto<'a>,
    {
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
    pub fn cur_bool<'a>(&'a self, t: &str, f: &str) -> Result<bool>
    where
        S: ArgInto<'a>,
    {
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
    pub fn cur_opt_bool<'a>(
        &'a self,
        t: &str,
        f: &str,
        n: &str,
    ) -> Result<Option<bool>>
    where
        S: ArgInto<'a>,
    {
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
    pub fn cur_key<'a, T: FromArg<'a>>(&'a self, sep: char) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    pub fn cur_val<'a, T: FromArg<'a>>(&'a self, sep: char) -> Result<T>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<Option<T>>
    where
        S: ArgInto<'a>,
    {
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
    ) -> Result<T>
    where
        S: ArgInto<'a>,
    {
        self.inner().cur_val_or_next(sep)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur_with<'a, T>(
        &'a self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()>
    where
        S: ArgInto<'a>,
    {
        self.inner().try_set_cur_with(res, f)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next_with<'a, T>(
        &'a mut self,
        res: &mut Option<T>,
        f: impl FnOnce(&'a str) -> Result<T>,
    ) -> Result<()>
    where
        S: ArgInto<'a>,
    {
        self.inner().try_set_next_with(res, f)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_cur<'a, T: FromArg<'a>>(
        &'a mut self,
        res: &mut Option<T>,
    ) -> Result<()>
    where
        S: ArgInto<'a>,
    {
        self.inner().try_set_cur(res)
    }

    /// Tries to set the value of `res` to some if it is none. Throws error if it
    /// is some.
    #[inline]
    pub fn try_set_next<'a, T: FromArg<'a>>(
        &'a mut self,
        res: &mut Option<T>,
    ) -> Result<()>
    where
        S: ArgInto<'a>,
    {
        self.inner().try_set_next(res)
    }

    /// Splits last argument by separator `sep` and parses each word into a
    /// resulting vector.
    ///
    /// Difference from [`Pareg::cur_list`] is that this will first to split
    /// and than try to parse.
    #[inline]
    pub fn split_cur<'a, T: FromArg<'a>>(&'a self, sep: &str) -> Result<Vec<T>>
    where
        S: ArgInto<'a>,
    {
        self.inner().split_cur(sep)
    }

    /// Parses multiple values in last argument separated by `sep`.
    ///
    /// Unlike [`Pareg::split_cur`], this will first try to parse and than
    /// check if the separator is present. So valid values may contain contents
    /// of `sep`, and it will properly parse the vales, whereas
    /// [`Pareg::split_cur`] would split `arg` and than try to parse.
    #[inline]
    pub fn cur_list<'a, T: FromRead>(&'a self, sep: &str) -> Result<Vec<T>>
    where
        S: ArgInto<'a>,
    {
        self.inner().cur_list(sep)
    }

    /// Splits next argument by separator `sep` and parses each word into a
    /// resulting vector.
    ///
    /// Difference from [`Pareg::next_list`] is that this will first to split
    /// and than try to parse.
    #[inline]
    pub fn split_next<'a, T: FromArg<'a>>(
        &'a mut self,
        sep: &str,
    ) -> Result<Vec<T>>
    where
        S: ArgInto<'a>,
    {
        self.inner().split_next(sep)
    }

    /// Parses multiple values in next argument separated by `sep`.
    ///
    /// Unlike [`Pareg::split_next`], this will first try to parse and than
    /// check if the separator is present. So valid values may contain contents
    /// of `sep`, and it will properly parse the vales, whereas
    /// [`Pareg::split_next`] would split `arg` and than try to parse.
    #[inline]
    pub fn next_list<'a, T: FromRead>(
        &'a mut self,
        sep: &str,
    ) -> Result<Vec<T>>
    where
        S: ArgInto<'a>,
    {
        self.inner().next_list(sep)
    }

    /// Leave parsing of the next arguments to the `FromArgs` implementation of
    /// `T`.
    pub fn next_sub<'a, T: FromArgs<'a>>(&'a mut self) -> Result<T>
    where
        S: ArgInto<'a>,
    {
        self.inner().next_sub()
    }

    /// Leave the parsing of the current and following arguments to the
    /// `FromArgs` implementation of `T`.
    pub fn cur_sub<'a, T: FromArgs<'a>>(&'a mut self) -> Result<T>
    where
        S: ArgInto<'a>,
    {
        self.inner().cur_sub()
    }

    /// Creates pretty error that the last argument (cur) is unknown.
    #[inline]
    pub fn err_unknown_argument<'a>(&'a self) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_unknown_argument()
    }

    /// Creates pretty error that there should be more arguments but there are
    /// no more arguments.
    #[inline]
    pub fn err_no_more_arguments<'a>(&'a self) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_no_more_arguments()
    }

    /// Creates error that sais that the current argument is specified too many
    /// times.
    #[inline]
    pub fn err_cur_too_many_arguments<'a>(&'a self) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_cur_too_many_arguments()
    }

    /// Creates error that says that the current argument has invalid value.
    #[inline]
    pub fn err_invalid<'a>(&'a self) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_invalid()
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_value<'a>(&'a self, value: String) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_invalid_value(value)
    }

    /// Creates error that says that the given part of the current argument has
    /// invalid value.
    #[inline]
    pub fn err_invalid_span<'a>(
        &'a self,
        span: impl RangeBounds<usize>,
    ) -> ArgError
    where
        S: ArgInto<'a>,
    {
        self.inner().err_invalid_span(span)
    }

    /// Adds additional information to error so that it has better error
    /// message. Consider using [`ParegRef::cur_manual`] or
    /// [`ParegRef::next_manual`] instead.
    #[inline]
    pub fn map_err<'a>(&'a self, err: ArgError) -> ArgError
    where
        S: ArgInto<'a>,
    {
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
    pub fn map_res<'a, T>(&'a self, res: Result<T>) -> Result<T>
    where
        S: ArgInto<'a>,
    {
        self.inner().map_res(res)
    }
}

impl Pareg<String> {
    /// Create [`Pareg`] from [`env::args`], the first argument is skipped.
    #[inline]
    pub fn args() -> Self {
        Self {
            args: env::args().collect(),
            cur: 1.into(),
        }
    }
}

impl Pareg<OsString> {
    /// Create [`Pareg`] from [`env::args`], the first argument is skipped.
    #[inline]
    pub fn args_os() -> Self {
        Self {
            args: env::args_os().collect(),
            cur: 1.into(),
        }
    }
}
