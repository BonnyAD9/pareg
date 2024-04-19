use crate::{
    by_ref::ByRef,
    err::{ArgError, Result},
    from_arg::FromArg,
};

/// An iterator over arguments. It can directly parse the value it yelds.
pub struct ArgIterator<'a, I>
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    iter: I,
    last: Option<&'a str>,
}

impl<'a, I> Iterator for ArgIterator<'a, I> where I: Iterator, I::Item: ByRef<&'a str> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.last = self.iter.next().by_ref();
        self.last
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
    pub fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>,
    {
        let last = self.last;
        if let Some(a) = self.next() {
            T::from_arg(a.by_ref())
        } else if let Some(last) = last {
            Err(ArgError::NoMoreArguments(Some(last.into())))
        } else {
            Err(ArgError::NoMoreArguments(None))
        }
    }
}

impl<'a, I> From<I> for ArgIterator<'a, I> where I: Iterator, I::Item: ByRef<&'a str> {
    fn from(value: I) -> Self {
        ArgIterator { iter: value, last: None }
    }
}
