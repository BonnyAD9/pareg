use std::borrow::Cow;

use crate::{ArgError, Reader, Result, SetFromRead, reader::ReadFmt};

/// Argument to [`parsef`] describing expected operation.
pub enum ParseFArg<'a, 'f> {
    /// Expect a string.
    Str(Cow<'a, str>),
    /// Expect to parse to the given value with the given format.
    Arg(&'a mut dyn SetFromRead, &'f ReadFmt<'f>),
}

/// Parsef implementation. Parse all data in `r` based on `args`.
///
/// This is usually used by the `parsef!` macro, but nothing forbids you from
/// constructing the parse operation at runtime.
pub fn parsef<'a, 'f>(
    r: &mut Reader,
    args: impl AsMut<[ParseFArg<'a, 'f>]>,
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
pub fn parsef_part<'a, 'f>(
    r: &mut Reader,
    mut args: impl AsMut<[ParseFArg<'a, 'f>]>,
) -> Result<Option<ArgError>> {
    let mut last_err = None;
    for a in args.as_mut() {
        last_err = match a {
            ParseFArg::Arg(a, fmt) => a.set_from_read(r, fmt)?,
            ParseFArg::Str(a) => {
                r.expect(a)?;
                None
            }
        };
    }

    Ok(last_err)
}
