use crate::{by_ref::ByRef, err::Result, from_arg::FromArg};

/// This trait represents a string reference object that can be parsed into a
/// type.
pub trait ArgInto<'a> {
    /// Parses this string into another type using the [`FromArg`] trait.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::ArgInto;
    ///
    /// assert_eq!("hello", "hello".arg_into::<&str>().unwrap());
    /// assert_eq!(5, "5".arg_into::<i32>().unwrap());
    /// ```
    fn arg_into<T>(self) -> Result<'a, T>
    where
        T: FromArg<'a>;
}

impl<'a, S> ArgInto<'a> for S
where
    S: ByRef<&'a str>,
{
    fn arg_into<T>(self) -> Result<'a, T>
    where
        T: FromArg<'a>,
    {
        T::from_arg(self.by_ref())
    }
}
