use std::borrow::Cow;

use crate::{ArgError, Reader, Result, SetFromRead};

/// Argument to [`parsef`] describing expected operation.
pub enum ParseFArg<'a> {
    /// Expect a string.
    Str(Cow<'a, str>),
    /// Expect to parse to the given value.
    Arg(&'a mut dyn SetFromRead),
}

/// Parsef implementation. Parse all data in `r` based on `args`.
///
/// This is usually used by the `parsef!` macro, but nothing forbids you from
/// constructing the parse operation at runtime.
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

/// Parsef part implementation. Parse part of data in `r`, based on `args`.
///
/// This is usually used by the `parsef_part!` macro, but nothing forbids you
/// from constructing the parse operation at runtime.
pub fn parsef_part<'a>(
    r: &mut Reader,
    mut args: impl AsMut<[ParseFArg<'a>]>,
) -> Result<Option<ArgError>> {
    let mut last_err = None;
    for a in args.as_mut() {
        last_err = match a {
            ParseFArg::Arg(a) => a.set_from_read(r)?,
            ParseFArg::Str(a) => {
                r.expect(a)?;
                None
            }
        };
    }

    Ok(last_err)
}
