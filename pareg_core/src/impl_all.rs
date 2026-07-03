/// Bulk implementation of a trait.
macro_rules! impl_all {
    (impl<$lt:lifetime>: $($t:ty),* $(,)? => $body:tt) => {
        $(impl<$lt> $t $body)*
    };
    (impl<$lt:lifetime> $tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl<$lt> $tr for $t $body)*
    };
    (impl $tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl $tr for $t $body)*
    };
}

pub(crate) use impl_all;
