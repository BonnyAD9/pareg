/// Bulk implementation of a trait.
#[macro_export]
macro_rules! impl_all {
    (impl<$lt:lifetime> $tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl<$lt> $tr for $t $body)*
    };
    ($tr:ty: $($t:ty),* $(,)? => $body:tt) => {
        $(impl $tr for $t $body)*
    };
}
