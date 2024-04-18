use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::impl_all::impl_all;

/// Similar to [`AsRef`], but this also gives the option to specify the lifetime
/// of the returned reference.
pub trait ByRef<T>
where
    T: ?Sized,
{
    /// Returns this as a reference to `T`.
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

impl<'a, R, T> ByRef<Option<&'a T>> for Option<R>
where
    R: ByRef<&'a T>,
    T: ?Sized,
{
    fn by_ref(self) -> Option<&'a T> {
        self.map(|a| a.by_ref())
    }
}
