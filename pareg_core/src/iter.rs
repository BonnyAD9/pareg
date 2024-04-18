use crate::{
    by_ref::ByRef, err::{ArgError, Result}, from_arg::FromArg
};

/// An iterator over arguments. It can directly parse the value it yelds.
pub trait ArgIterator<'a>: Iterator
where
    Self::Item: ByRef<&'a str>,
{
    /// Parses the next value in the iterator.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgIterator;
    ///
    /// let args = ["hello", "10", "0.25", "always"];
    /// let mut args = args.iter();
    ///
    /// assert_eq!("hello", args.next_arg::<&str>().unwrap());
    /// assert_eq!(10, args.next_arg::<usize>().unwrap());
    /// assert_eq!(0.25, args.next_arg::<f64>().unwrap());
    /// ```
    fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>;
}

impl<'a, I> ArgIterator<'a> for I
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
{
    fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>,
    {
        if let Some(a) = self.next() {
            T::from_arg(a.by_ref())
        } else {
            Err(ArgError::NoMoreArguments(None))
        }
    }
}
