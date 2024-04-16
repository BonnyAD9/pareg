use std::{
    any::type_name,
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::Display,
    net::{
        IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6,
    },
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::err::{ArgError, Result};

pub trait FromArg<'a>: Sized {
    fn from_arg(arg: &'a str) -> Result<Self>;
}

pub trait FromArgStr: FromStr {}

macro_rules! impl_all {
    ($tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl $tr for $t $body)*
    };
}

impl<'a, T> FromArg<'a> for T
where
    T: FromArgStr,
    T::Err: Display,
{
    fn from_arg(arg: &'a str) -> Result<'a, Self> {
        T::from_str(arg).map_err(|e| ArgError::FailedToParse {
            typ: type_name::<T>(),
            value: arg.into(),
            msg: Some(format!("{e}").into()),
        })
    }
}

impl_all! { FromArgStr:
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64, usize, isize,
    bool, char, String, PathBuf, OsString, IpAddr, SocketAddr, Ipv4Addr,
    Ipv6Addr, SocketAddrV4, SocketAddrV6,
    => {}
}

impl<'a> FromArg<'a> for &'a str {
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(arg)
    }
}

impl<'a> FromArg<'a> for &'a Path {
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(Path::new(arg))
    }
}

impl<'a> FromArg<'a> for &'a OsStr {
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(OsStr::new(arg))
    }
}

impl<'a> FromArg<'a> for Cow<'a, str> {
    fn from_arg(arg: &'a str) -> Result<Self> {
        Ok(arg.into())
    }
}

impl<'a, T> FromArg<'a> for Option<T>
where
    T: FromArg<'a>,
{
    fn from_arg(arg: &'a str) -> Result<Self> {
        if arg.is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_arg(arg)?))
        }
    }
}
