use std::{
    ffi::OsString,
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

use crate::{parsef_part, reader::Reader, ArgError, ParseFArg, Result};

/// Trait similar to [`crate::FromArg`]. Difference is that this may parse only
/// part of the input.
pub trait FromRead: Sized {
    /// Parses part of the input from the reader. On failure returns Err. On
    /// success returns the parsed value, and optionally also error that would
    /// occur if more of the input was expected to be parsed. If this returns
    /// successfully and there is no error, it usually means that all of the
    /// input from reader was consumed.
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)>;
}

macro_rules! impl_from_read {
    ($($(-$it:ident)? $($ut:ident)?),* $(,)?) => {
        $(impl FromRead for $($it)? $($ut)? {
            fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
                const RADIX: u32 = 10;
                let mut res: Self = 0;
                let start_pos = r.pos();

                macro_rules! unwrap_or_exit {
                    ($v:expr, $msg:literal) => {
                        match $v {
                            Some(v) => v,
                            None => return Ok((res, Some(r.err_parse($msg)))),
                        }
                    };
                }

                macro_rules! pass_or_exit {
                    ($v:expr) => {
                        match $v {
                            Ok(r) => r,
                            Err(e) => return Ok((res, Some(e))),
                        }
                    };
                }

                macro_rules! loop_signed {
                    ($op:ident, $ignore:ident) => {

                        while let Some(c) = r.peek().transpose() {
                            let r2 = res.checked_mul(RADIX as Self);
                            let d = pass_or_exit!(c);
                            let d = unwrap_or_exit!(
                                d.to_digit(RADIX),
                                "Invalid digit in string."
                            );
                            res = pass_or_exit!(
                                r2.and_then(|r| r.$op(d as Self)).ok_or_else(||
                                    r.err_parse(
                                        "Number doesn't fit the target type."
                                    ).span_start(start_pos.unwrap_or_default())
                                        .hint(format!(
                                            "Value must be in range from `{}` \
                                            to `{}`.",
                                            Self::MIN,
                                            Self::MAX
                                        ))
                                )
                            );
                            _ = r.next();
                        }
                    };
                }

                $(
                    if matches!(pass_or_exit!(r.peek()), Some('-')) {
                        pass_or_exit!(r.next().transpose());
                        loop_signed!(checked_sub, $it);
                    } else {
                        loop_signed!(checked_add, $it);
                    }
                )?

                $(loop_signed!(checked_add, $ut);)?

                if start_pos == r.pos() {
                    Err(r.err_parse("Expected at least one digit."))
                } else {
                    Ok((res, None))
                }
            }
        })*
    };
}

impl_from_read!(u8, u16, u32, u64, usize, -i8, -i16, -i32, -i64, -isize);

/// Implements [`std::str::FromStr`] for type that implements [`FromRead`].
///
/// In future this may be deprecated in favor of derive macro.
#[macro_export]
macro_rules! impl_from_str_with_read {
    ($($typ:ident)::*$(<$($gen:tt),*?> $(where $($con:tt),*)?)?) => {
        impl$(<$($gen),*>)? std::str::FromStr for $($typ)::*$(<$($gen),*>
        $(where $($con),*)?)?
        {
            type Err = $crate::ArgError;

            fn from_str(s: &str) -> $crate::Result<Self> {
                use $crate::FromRead;
                let (val, err) = Self::from_read(&mut s.into())?;
                if let Some(err) = err {
                    Err(err)
                } else {
                    Ok(val)
                }
            }
        }
    };
}

/// Implements [`std::str::FromStr`] and [`crate::FromArg`] for type that
/// implements [`FromRead`].
///
/// In future this may be deprecated in favor of derive macros.
#[macro_export]
macro_rules! impl_from_arg_str_with_read {
    ($($typ:ident)::*$(<$($gen:tt),*?> $(where $($con:tt),*)?)?) => {
        impl$(<$($gen),*>)? std::str::FromStr for $($typ)::*$(<$($gen),*>
        $(where $($con),*)?)?
        {
            type Err = $crate::ArgError;

            fn from_str(s: &str) -> $crate::Result<Self> {
                use $crate::FromRead;
                let (val, err) = Self::from_read(&mut s.into())?;
                if let Some(err) = err {
                    Err(err)
                } else {
                    Ok(val)
                }
            }
        }

        impl$(<$($gen),*>)? $crate::FromArgStr for $($typ)::*$(<$($gen),*>
        $(where $($con),*)?)?
        {
        }
    };
}

impl FromRead for bool {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let (c, _) = char::from_read(r)?;
        match c {
            't' => {
                r.expect("rue")?;
                Ok((true, None))
            }
            'f' => {
                r.expect("alse")?;
                Ok((false, None))
            }
            c => Err(r.err_parse(format!(
                "Expected `true` or `false`, but there is `{c}`"
            ))),
        }
    }
}

impl FromRead for char {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let Some(c) = r.next().transpose()? else {
            return Err(r.err_parse("Expected character."));
        };
        Ok((c, None))
    }
}

impl FromRead for String {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let mut res = String::new();
        r.read_all(&mut res)?;
        Ok((res, None))
    }
}

impl FromRead for PathBuf {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let mut res = String::new();
        r.read_all(&mut res)?;
        Ok((res.into(), None))
    }
}

impl FromRead for OsString {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let mut res = String::new();
        r.read_all(&mut res)?;
        Ok((res.into(), None))
    }
}

impl FromRead for Ipv4Addr {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let mut c: (u8, u8, u8, u8) = Default::default();
        let r = parsef_part(
            r,
            [
                ParseFArg::Arg(&mut c.0),
                ParseFArg::Str(".".into()),
                ParseFArg::Arg(&mut c.1),
                ParseFArg::Str(".".into()),
                ParseFArg::Arg(&mut c.2),
                ParseFArg::Str(".".into()),
                ParseFArg::Arg(&mut c.3),
            ],
        )?;
        Ok((Ipv4Addr::new(c.0, c.1, c.2, c.3), r))
    }
}

impl FromRead for SocketAddrV4 {
    fn from_read(r: &mut Reader) -> Result<(Self, Option<ArgError>)> {
        let mut adr: Ipv4Addr = Ipv4Addr::LOCALHOST;
        let mut port: u16 = 0;
        let r = parsef_part(
            r,
            [
                ParseFArg::Arg(&mut adr),
                ParseFArg::Str(":".into()),
                ParseFArg::Arg(&mut port),
            ],
        )?;
        Ok((SocketAddrV4::new(adr, port), r))
    }
}
