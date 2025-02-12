use std::borrow::Cow;

use crate::{ArgError, FromRead, Reader, Result};

pub trait SetFromRead {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>>;
}

impl<T: FromRead> SetFromRead for T {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>> {
        let start = r.pos().unwrap_or_default();
        let res = Self::from_read(r);
        if let Some(v) = res.res {
            *self = v;
            Ok(res.err)
        } else {
            Err(res.err.unwrap_or_else(|| {
                r.err_parse("Failed to parse argument.").span_start(start)
            }))
        }
    }
}

pub enum ParseFArg<'a> {
    Str(Cow<'a, str>),
    Arg(&'a mut dyn SetFromRead),
}

pub fn parsef<'a>(
    r: &mut Reader,
    args: impl AsMut<[ParseFArg<'a>]>,
) -> Result<()> {
    let res = parsef_part(r, args)?;
    if r.peek()?.is_none() {
        Ok(())
    } else {
        Err(res.unwrap_or_else(|| r.err_parse("Unused input")))
    }
}

pub fn parsef_part<'a>(
    r: &mut Reader,
    mut args: impl AsMut<[ParseFArg<'a>]>,
) -> Result<Option<ArgError>> {
    let mut last_err = None;
    for a in args.as_mut() {
        last_err = match a {
            ParseFArg::Arg(a) => a.set_from_read(r)?,
            ParseFArg::Str(a) => {
                match_prefix(a, r)?;
                None
            }
        };
    }

    Ok(last_err)
}

pub fn match_prefix(prefix: &str, r: &mut Reader) -> Result<()> {
    // TODO better error on first fail
    for p in prefix.chars() {
        let Some(s) = r.next().transpose()? else {
            return r
                .err_parse("Unexpected end of string.")
                .inline_msg(format!("Expected `{p}`"))
                .err();
        };
        if p != s {
            return r
                .err_parse(format!("Unexpected character `{s}`."))
                .inline_msg(format!("Expected `{p}`."))
                .err();
        }
    }
    Ok(())
}
