/*use std::{borrow::Cow, str::FromStr};

use crate::{ArgError, FromArg, Result};

pub trait ParseF {
    fn set_from_arg(&mut self, arg: &str) -> Result<()>;
}

impl<T: FromArg> ParseF for T {
    fn set_from_arg(&mut self, arg: &str) -> Result<()> {
        *self = T::from_arg(arg)?;
        Ok(())
    }
}

pub enum ParseFArg<'a> {
    Str(Cow<'a, str>),
    Arg(&'a mut dyn ParseF),
}

#[macro_export]
macro_rules! parsef_args {
    ($s:expr, $l0:expr $(, $e:expr, $l:expr)+) => {
        $crate::parsef([$crate::ParseFArg::Str($l0.into()), $($crate::ParseFArg::Arg($e), $crate::ParseFArg::Str($l.into()),)+], $s)
    };
    ($s:expr $(, $l:expr, $e:expr)+) => {
        $crate::parsef([$($crate::ParseFArg::Str($l.into()), $crate::ParseFArg::Arg($e),)+], $s)
    };
}

pub fn parsef<'a>(
    mut args: impl AsMut<[ParseFArg<'a>]>,
    s: impl AsRef<str>,
) -> Result<()> {
    let mut s = s.as_ref();
    let s0 = s;
    let mut args = args.as_mut().iter_mut().peekable();
    while let Some(a) = args.next() {
        let arg = match a {
            ParseFArg::Arg(a) => a,
            ParseFArg::Str(a) => {
                match_prefix(a, s).map_err(|e| e.postfix_of(s0.into()))?;
                s = &s[a.len()..];
                continue;
            }
        };

        let Some(pk) = args.peek() else {
            arg.set_from_arg(s)?;
            s = &s[s.len()..];
            continue;
        };

        let ParseFArg::Str(pk) = pk else {
            return ArgError::parse_msg("Invalid parsef args. There is no matching pattern between two args.", s0.into()).err();
        };

        let Some(idx) = s.find(pk.as_ref()) else {
            return ArgError::parse_msg(
                format!("Couldn't find the pattern `{pk}` in the string."),
                s0.into(),
            )
            .postfix_of(pk.as_ref().into())
            .err();
        };

        arg.set_from_arg(&s[..idx])?;
        s = &s[idx..];
    }

    if s.is_empty() {
        Ok(())
    } else {
        return ArgError::parse_msg("Unused input.", s.into())
            .postfix_of(s0.into())
            .err();
    }
}

pub fn match_prefix(prefix: &str, s: &str) -> Result<()> {
    for ((i, p), s) in prefix.char_indices().zip(s.chars()) {
        if p != s {
            return ArgError::parse_msg(
                "Unexpected character.",
                s.to_string(),
            )
            .spanned(i..i + s.len_utf8())
            .inline_msg(format!("Expected `{p}`."))
            .err();
        }
    }
    Ok(())
}
*/
