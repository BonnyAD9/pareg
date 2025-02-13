use crate::{ArgError, FromRead, Reader, Result};

pub trait SetFromRead {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>>;
}

impl<T: FromRead> SetFromRead for T {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>> {
        let err;
        (*self, err) = Self::from_read(r)?;
        Ok(err)
    }
}
