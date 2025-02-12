use crate::{ParseF, Result};

pub struct CheckRef<'a, T: ParseF, F: Fn(&T) -> Result<()>>(
    pub &'a mut T,
    pub F,
);

impl<T: ParseF, F: Fn(&T) -> Result<()>> ParseF for CheckRef<'_, T, F> {
    fn set_from_read(
        &mut self,
        r: &mut crate::Reader,
    ) -> Result<Option<crate::ArgError>> {
        match self.0.set_from_read(r) {
            Ok(r) => self.1(self.0).map(|_| r),
            e => e,
        }
    }
}
