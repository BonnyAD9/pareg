use crate::{Reader, Result, SetFromRead};

/// Wraps [`SetFromRead`] implementation of type so that it also checks for
/// valid values with the given function.
pub struct CheckRef<
    'a,
    T: SetFromRead,
    F: Fn(&Reader, usize, &T) -> Result<()>,
>(pub &'a mut T, pub F);

impl<T: SetFromRead, F: Fn(&Reader, usize, &T) -> Result<()>> SetFromRead
    for CheckRef<'_, T, F>
{
    fn set_from_read(
        &mut self,
        r: &mut Reader,
    ) -> Result<Option<crate::ArgError>> {
        let pos = r.pos().unwrap_or_default();
        match self.0.set_from_read(r) {
            Ok(res) => self.1(r, pos, self.0).map(|_| res),
            e => e,
        }
    }
}
