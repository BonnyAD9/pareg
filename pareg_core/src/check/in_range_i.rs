use std::str::FromStr;

use crate::{ArgErrCtx, ArgError, FromArg, FromArgStr};

pub struct InRangeI<
    T: PartialOrd<i128> + for<'a> FromArg<'a>,
    const START: i128,
    const END: i128,
>(pub T);

impl<
        T: PartialOrd<i128> + for<'a> FromArg<'a>,
        const START: i128,
        const END: i128,
    > FromStr for InRangeI<T, START, END>
{
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let r = T::from_arg(s)?;
        if r < START || r > END {
            Err(ArgError::InvalidValue(ArgErrCtx::from_msg(format!("Invalid value. Value must be in rang from {START} to {END}"), s.to_string()).into()))
        } else {
            Ok(Self(r))
        }
    }
}

impl<
        T: PartialOrd<i128> + for<'a> FromArg<'a>,
        const START: i128,
        const END: i128,
    > FromArgStr for InRangeI<T, START, END>
{
}
