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

use thiserror::Error;

type Result<T> = std::result::Result<T, ArgError>;

#[derive(Debug, Error)]
enum ArgError {
    #[error("Expected next argument")]
    NoMoreArguments,
    #[error(
        "Failed to parse '{value}' into '{typ}'{}",
        if let Some(msg) = .msg {
            format!(": {msg}")
        } else {
            "".to_owned()
        }
    )]
    FailedToParse {
        typ: &'static str,
        value: String,
        msg: Option<String>,
    },
}

trait ArgRefIterator<'a> {
    fn next_arg<T>(&mut self) -> Result<T>
    where
        T: FromArg<'a>;
}

trait FromArg<'a>: Sized {
    fn from_arg(arg: &'a str) -> Result<Self>;
}

trait FromArgStr: FromStr {}

impl<'a, I> ArgRefIterator<'a> for I
where
    I: Iterator<Item = &'a str>,
{
    fn next_arg<T>(&mut self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        if let Some(a) = self.next() {
            T::from_arg(a)
        } else {
            Err(ArgError::NoMoreArguments)
        }
    }
}

impl<'a, T> FromArg<'a> for T
where
    T: FromArgStr,
    T::Err: Display,
{
    fn from_arg(arg: &'a str) -> Result<Self> {
        T::from_str(arg).map_err(|e| ArgError::FailedToParse {
            typ: type_name::<T>(),
            value: arg.to_owned(),
            msg: Some(format!("{e}")),
        })
    }
}

macro_rules! impl_all {
    ($tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl $tr for $t $body)*
    };
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
