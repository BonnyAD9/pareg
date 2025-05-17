use crate::{Reader, Result, SetFromRead, reader::ReadFmt};

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
    fn set_from_read<'a>(
        &mut self,
        r: &mut Reader,
        fmt: &'a ReadFmt<'a>,
    ) -> Result<Option<crate::ArgError>> {
        let pos = r.pos();
        match self.0.set_from_read(r, fmt) {
            Ok(res) => self.1(r, pos, self.0).map(|_| res),
            e => e,
        }
    }
}
