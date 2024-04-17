use crate::{err::Result, from_arg::FromArg};

pub trait ArgInto<'a, T> {
    fn arg_into(self) -> Result<'a, T>;
}

impl<'a, T> ArgInto<'a, T> for &'a str
where
    T: FromArg<'a>,
{
    fn arg_into(self) -> Result<'a, T> {
        T::from_arg(self)
    }
}

impl<'a, T> ArgInto<'a, T> for &'a String
where
    T: FromArg<'a>,
{
    fn arg_into(self) -> Result<'a, T> {
        T::from_arg(self)
    }
}
