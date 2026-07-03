use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    rc::Rc,
    sync::Arc,
};

use crate::{err::Result, from_arg::FromArg, impl_all::impl_all};

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
    fn arg_into<T>(&'a self) -> Result<T>
    where
        T: FromArg<'a>;
}

impl_all! { impl<'a> ArgInto<'a>:
    OsStr, OsString, Arc<OsStr>, Rc<OsStr>, Cow<'a, OsStr>, &OsStr => {
        #[inline(always)]
        fn arg_into<T>(&'a self) -> Result<T>
        where
            T: FromArg<'a>,
        {
            #[allow(clippy::useless_asref)]
            T::from_os_arg((*self).as_ref())
        }
    }
}

impl_all! { impl<'a> ArgInto<'a>:
    str, String, Arc<str>, Rc<str>, Cow<'a, str>, &str => {
        #[inline(always)]
        fn arg_into<T>(&'a self) -> Result<T>
        where
            T: FromArg<'a>,
        {
            #[allow(clippy::useless_asref)]
            T::from_arg((*self).as_ref())
        }
    }
}
