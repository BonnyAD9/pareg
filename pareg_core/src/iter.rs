use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::{
    err::{ArgError, Result},
    from_arg::FromArg,
    impl_all,
};

// Represents reference to an object with a specific lifetime.
pub trait ByRef<'a, T>
where
    T: ?Sized,
{
    fn by_ref(self) -> &'a T;
}

impl_all! {
    impl<'a> ByRef<'a, str>:
        &'a str, &'a String, &'a Arc<str>, &'a Rc<str>, &'a Cow<'a, str>,
        &&'a str
    => {
        fn by_ref(self) -> &'a str {
            #[allow(clippy::useless_asref)]
            (*self).as_ref()
        }
    }
}

/// An iterator over arguments. It can directly parse the value it yelds.
pub trait ArgIterator<'a>: Iterator
where
    Self::Item: ByRef<'a, str>,
{
    fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>;
}

impl<'a, I> ArgIterator<'a> for I
where
    I: Iterator,
    I::Item: ByRef<'a, str>,
{
    fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>,
    {
        if let Some(a) = self.next() {
            T::from_arg(a.by_ref())
        } else {
            Err(ArgError::NoMoreArguments(None))
        }
    }
}
