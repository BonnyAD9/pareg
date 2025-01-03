use crate::{reader::Reader, ArgError};

pub struct ParseResult<T> {
    pub err: Option<ArgError>,
    pub res: Option<T>,
}

pub trait FromRead: Sized {
    fn from_read(r: &mut Reader) -> ParseResult<Self>;
}

macro_rules! impl_from_read {
    ($($(-$it:ident)? $($ut:ident)?),* $(,)?) => {
        $(impl FromRead for $($it)? $($ut)? {
            fn from_read(r: &mut Reader) -> ParseResult<Self> {
                const RADIX: u32 = 10;
                let mut res: Self = 0;
                let start_pos = r.pos().unwrap_or_default();

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

                        while let Some(c) = r.next() {
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
                                    ).span_start(start_pos)
                                        .hint(format!(
                                            "Value must be in range from `{}` \
                                            to `{}`.",
                                            Self::MIN,
                                            Self::MAX
                                        ))
                                )
                            );
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

                ParseResult { err: None, res: Some(res) }
            }
        })*
    };
}

impl_from_read!(u8, u16, u32, u64, usize, -i8, -i16, -i32, -i64, -isize);
