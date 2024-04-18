use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::{
    err::{ArgError, Result},
    from_arg::FromArg,
    impl_all,
};

// Represents reference to an object with a specific lifetime.
pub trait ByRef<T>
where
    T: ?Sized,
{
    fn by_ref(self) -> T;
}

impl_all! {
    impl<'a> ByRef<&'a str>:
        &'a str, &'a String, &'a Arc<str>, &'a Rc<str>, &'a Cow<'a, str>,
        &&'a str
    => {
        fn by_ref(self) -> &'a str {
            #[allow(clippy::useless_asref)]
            (*self).as_ref()
        }
    }
}

impl<'a, R, T> ByRef<Option<&'a T>> for Option<R> where R: ByRef<&'a T>, T: ?Sized {
    fn by_ref(self) -> Option<&'a T> {
        self.map(|a| a.by_ref())
    }
}

/// An iterator over arguments. It can directly parse the value it yelds.
pub trait ArgIterator<'a>: Iterator
where
    Self::Item: ByRef<&'a str>,
{
    fn next_arg<T>(&mut self) -> Result<'a, T>
    where
        T: FromArg<'a>;
}

impl<'a, I> ArgIterator<'a> for I
where
    I: Iterator,
    I::Item: ByRef<&'a str>,
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
