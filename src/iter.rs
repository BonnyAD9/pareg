use crate::{
    err::{ArgError, Result},
    from_arg::FromArg,
};

trait ArgIterator<'a> {
    fn next_arg<T>(&mut self) -> Result<T>
    where
        T: FromArg<'a>;
}

impl<'a, I> ArgIterator<'a> for I
where
    I: Iterator<Item = &'a str>,
{
    fn next_arg<T>(&mut self) -> Result<T>
    where
        T: FromArg<'a>,
    {
        if let Some(a) = self.next() {
            T::from_arg(a)
        } else {
            Err(ArgError::NoMoreArguments)
        }
    }
}
