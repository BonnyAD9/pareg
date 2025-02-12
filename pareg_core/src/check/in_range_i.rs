use std::str::FromStr;

use crate::{ArgErrCtx, ArgError, FromArg, FromArgStr};

pub struct InRangeI<
    T: TryInto<i128> + Copy + for<'a> FromArg<'a>,
    const START: i128,
    const END: i128,
>(pub T);

impl<
        T: TryInto<i128> + Copy + for<'a> FromArg<'a>,
        const START: i128,
        const END: i128,
    > FromStr for InRangeI<T, START, END>
{
    type Err = ArgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || {
            ArgError::InvalidValue(
                ArgErrCtx::from_msg(
                    format!(
                        "Invalid value. Value must be in rang from \
                        {START} to {END}."
                    ),
                    s.to_string(),
                )
                .into(),
            )
        };
        let r = T::from_arg(s)?;
        let Ok(rv) = r.try_into() else {
            return Err(err());
        };
        if rv < START || rv >= END {
            Err(err())
        } else {
            Ok(Self(r))
        }
    }
}

impl<
        T: TryInto<i128> + Copy + for<'a> FromArg<'a>,
        const START: i128,
        const END: i128,
    > FromArgStr for InRangeI<T, START, END>
{
}
