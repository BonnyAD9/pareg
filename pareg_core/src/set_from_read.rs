use crate::{ArgError, FromRead, Reader, Result};

pub trait SetFromRead {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>>;
}

impl<T: FromRead> SetFromRead for T {
    fn set_from_read(&mut self, r: &mut Reader) -> Result<Option<ArgError>> {
        let start = r.pos().unwrap_or_default();
        let res = Self::from_read(r);
        if let Some(v) = res.res {
            *self = v;
            Ok(res.err)
        } else {
            Err(res.err.unwrap_or_else(|| {
                r.err_parse("Failed to parse argument.").span_start(start)
            }))
        }
    }
}
