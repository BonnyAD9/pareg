use crate::{
    bool_arg,
    by_ref::ByRef,
    err::{ArgError, Result},
    from_arg::FromArg,
    key_arg, key_mval_arg, key_val_arg, mval_arg, opt_bool_arg, val_arg,
};

/// An iterator over arguments. It can directly parse the value it yelds.
pub struct ArgIterator<'a, I>
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    iter: I,
    cur: Option<&'a str>,
}

impl<'a, I> Iterator for ArgIterator<'a, I>
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    type Item = &'a str;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = self.iter.next().by_ref();
        self.cur
    }
}

impl<'a, I> From<I> for ArgIterator<'a, I>
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    fn from(value: I) -> Self {
        ArgIterator {
            iter: value,
            cur: None,
        }
    }
}

impl<'a, I> ArgIterator<'a, I>
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    /// Parses the next value in the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["hello", "10", "0.25", "always"];
    /// let mut args: ArgIterator<_> = args.iter().into();
    ///
    /// assert_eq!("hello", args.next_arg::<&str>().unwrap());
    /// assert_eq!(10, args.next_arg::<usize>().unwrap());
    /// assert_eq!(0.25, args.next_arg::<f64>().unwrap());
    /// ```
    #[inline]
    pub fn next_arg<T>(&mut self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        let last = self.cur;
        if let Some(a) = self.next() {
            T::from_arg(a.by_ref())
        } else if let Some(last) = last {
            Err(ArgError::NoMoreArguments(Some(last.to_owned().into())))
        } else {
            Err(ArgError::NoMoreArguments(None))
        }
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn next_key_mval<K, V>(&mut self, sep: char) -> Result<(K, Option<V>)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        key_mval_arg(self.next_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn next_key_val<K, V>(&mut self, sep: char) -> Result<(K, V)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        key_val_arg(self.next_arg()?, sep)
    }

    /// Uses the function [`bool_arg`] on the next value.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["true", "yes", "never"];
    /// let mut args: ArgIterator<_> = args.iter().into();
    ///
    /// assert_eq!(true, args.next_bool("true", "false").unwrap());
    /// assert_eq!(true, args.next_bool("yes", "no").unwrap());
    /// assert_eq!(false, args.next_bool("always", "never").unwrap());
    /// ```
    #[inline(always)]
    pub fn next_bool(&mut self, t: &str, f: &str) -> Result<bool> {
        bool_arg(t, f, self.next_arg()?)
    }

    /// Uses the function [`opt_bool_arg`] on the next argument.
    ///
    /// Parse bool value in a specific way. If the value of lowercase `arg` is
    /// equal to `t` returns true, if it is equal to `f` returns false and
    /// if it is equal to `n` returns [`None`]. Otherwise returns error.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["always", "never", "auto"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
        opt_bool_arg(t, f, n, self.next_arg()?)
    }

    /// Uses the function [`key_arg`] on the next value.
    ///
    /// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
    ///
    /// In case that there is no `'='`, parses the whole input.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn next_key<T>(&mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        key_arg(self.next_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn next_val<T>(&mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        val_arg(self.next_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn next_mval<T>(&mut self, sep: char) -> Result<Option<T>>
    where
        T: FromArg<'a>,
    {
        mval_arg(self.next_arg()?, sep)
    }

    /// Parses the last returned value from the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["hello", "10", "0.25", "always"];
    /// let mut args: ArgIterator<_> = args.iter().into();
    ///
    /// args.next();
    /// assert_eq!("hello", args.cur_arg::<&str>().unwrap());
    /// args.next();
    /// assert_eq!(10, args.cur_arg::<usize>().unwrap());
    /// args.next();
    /// assert_eq!(0.25, args.cur_arg::<f64>().unwrap());
    /// ```
    #[inline(always)]
    pub fn cur_arg<T>(&self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        if let Some(arg) = self.cur {
            T::from_arg(arg)
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
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn cur_key_mval<K, V>(&self, sep: char) -> Result<(K, Option<V>)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        key_mval_arg(self.cur_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn cur_key_val<K, V>(&self, sep: char) -> Result<(K, V)>
    where
        K: FromArg<'a>,
        V: FromArg<'a>,
    {
        key_val_arg(self.cur_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["true", "yes", "never"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
        bool_arg(t, f, self.cur_arg()?)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["always", "never", "auto"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
        opt_bool_arg(t, f, n, self.cur_arg()?)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn cur_key<T>(&mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        key_arg(self.cur_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn cur_val<T>(&mut self, sep: char) -> Result<T>
    where
        T: FromArg<'a>,
    {
        val_arg(self.cur_arg()?, sep)
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
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["key=value", "5:0.25", "only_key"];
    /// let mut args: ArgIterator<_> = args.iter().into();
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
    pub fn cur_mval<T>(&mut self, sep: char) -> Result<Option<T>>
    where
        T: FromArg<'a>,
    {
        mval_arg(self.cur_arg()?, sep)
    }
}
