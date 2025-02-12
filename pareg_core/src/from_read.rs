use crate::{reader::Reader, ArgError, Result};

/// Result of [`FromRead`] operation.
///
/// Contains optional value and optional error (both may be present). If both
/// value and error is present, the error represents error that would occur if
/// more of the input was expected to be parsed.
///
/// If `err` is [`None`] than `res` should always be [`Some`], but faulty
/// implementor may return such result.
pub struct ParseResult<T> {
    /// Error of the parse operation.
    ///
    /// When `res` is [`None`], parsing failed and this is the error. If `res`
    /// is [`Some`], this is error that would occur if more of the input is
    /// expected to be consumed.
    ///
    /// If this is [`None`], it usualy means that all of the input was
    /// consumed, and it was parsed successfully.
    ///
    /// This should never be [`None`] if `res` is [`None`].
    pub err: Option<ArgError>,
    /// Result of the parse operation.
    ///
    /// If this is [`None`], parsing has failed and `err` contains the error.
    /// Otherwise parsing was successfull. `err` can also contain error that
    /// would occur if more of the input was expected to be parsed.
    ///
    /// If this is [`None`], `err` should never be [`None`].
    pub res: Option<T>,
}

/// Trait similar to [`crate::FromArg`]. Difference is that this may parse only
/// part of the input.
pub trait FromRead: Sized {
    /// Parses part of the input from the reader. See [`ParseResult`] for more
    /// info about how to interpret the result.
    fn from_read(r: &mut Reader) -> ParseResult<Self>;
}

macro_rules! impl_from_read {
    ($($(-$it:ident)? $($ut:ident)?),* $(,)?) => {
        $(impl FromRead for $($it)? $($ut)? {
            fn from_read(r: &mut Reader) -> ParseResult<Self> {
                const RADIX: u32 = 10;
                let mut res: Self = 0;
                let start_pos = r.pos();

                macro_rules! unwrap_or_exit {
                    ($v:expr, $msg:literal) => {
                        match $v {
                            Some(v) => v,
                            None => return ParseResult {
                                err: Some(r.err_parse($msg)),
                                res: Some(res),
                            }
                        }
                    };
                }

                macro_rules! pass_or_exit {
                    ($v:expr) => {
                        match $v {
                            Ok(r) => r,
                            Err(e) => return ParseResult {
                                err: Some(e),
                                res: Some(res),
                            }
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

                ParseResult {
                    err: None,
                    res: (start_pos != r.pos())
                        .then_some(res)
                }
            }
        })*
    };
}

impl_from_read!(u8, u16, u32, u64, usize, -i8, -i16, -i32, -i64, -isize);

impl<T> ParseResult<T> {
    pub fn new(v: T, res: Result<Option<ArgError>>) -> Self {
        match res {
            Ok(err) => Self::success(v, err),
            Err(err) => Self::failure(err),
        }
    }

    pub fn success(v: T, err: Option<ArgError>) -> Self {
        Self { res: Some(v), err }
    }

    pub fn failure(err: ArgError) -> Self {
        Self {
            res: None,
            err: Some(err),
        }
    }
}
