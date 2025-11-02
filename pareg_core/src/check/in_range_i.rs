use std::str::FromStr;

use crate::{ArgError, FromArg, FromArgStr};

/// Wraps type, so that its [`FromArg`] implementation also checks that the
/// given value is in the range given by const parameters.
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
            ArgError::invalid_value(
                format!(
                    "Invalid value. Value must be in range from \
                    {START} to {END}."
                ),
                s,
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
