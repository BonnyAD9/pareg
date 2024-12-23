use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    net::{
        IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6,
    },
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
    sync::Arc,
};

use crate::{
    err::{ArgError, Result},
    impl_all::impl_all,
    ArgErrCtx,
};

/// Represents a trait similar to [`FromStr`], in addition it may return type
/// that references the original string slice. If your type already implements
/// [`FromStr`], you can just implement [`FromArgStr`].
pub trait FromArg<'a>: Sized {
    /// Parses the string into `Self`.
    ///
    /// # Examples
    /// ```rust
    /// use pareg_core::FromArg;
    ///
    /// assert_eq!("hello", <&str>::from_arg("hello").unwrap());
    /// assert_eq!("hello".to_owned(), String::from_arg("hello").unwrap());
    /// assert_eq!(5, i32::from_arg("5").unwrap());
    /// ```
    fn from_arg(arg: &'a str) -> Result<Self>;
}

/// Default implementation for [`FromArg`] for types that implement [`FromStr`]
pub trait FromArgStr: FromStr<Err = ArgError> {}

impl<T> FromArg<'_> for T
where
    T: FromArgStr,
{
    #[inline]
    fn from_arg(arg: &str) -> Result<Self> {
        T::from_str(arg)
    }
}

impl_all! { impl<'a> FromArg<'a>:
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64, usize, isize,
    bool, char, String, PathBuf, OsString, IpAddr, SocketAddr, Ipv4Addr,
    Ipv6Addr, SocketAddrV4, SocketAddrV6,
    => {
        #[inline(always)]
        fn from_arg(arg: &'a str) -> Result<Self> {
            Self::from_str(arg).map_err(|e| {
                ArgError::FailedToParse(Box::new(
                    ArgErrCtx::from_inner(e, arg.to_string())
                ))
            })
        }
    }
}

impl<'a> FromArg<'a> for &'a str {
    #[inline(always)]
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(arg)
    }
}

impl<'a> FromArg<'a> for &'a Path {
    #[inline(always)]
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(Path::new(arg))
    }
}

impl<'a> FromArg<'a> for &'a OsStr {
    #[inline(always)]
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(OsStr::new(arg))
    }
}

impl_all! {
    impl<'a> FromArg<'a>: Arc<str>, Rc<str>, Cow<'a, str> => {
        #[inline(always)]
        fn from_arg(arg: &'a str) -> Result<Self> {
            Ok(arg.into())
        }
    }
}

impl<'a, T> FromArg<'a> for Option<T>
where
    T: FromArg<'a>,
{
    #[inline]
    fn from_arg(arg: &'a str) -> Result<Self> {
        if arg.is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_arg(arg)?))
        }
    }
}
