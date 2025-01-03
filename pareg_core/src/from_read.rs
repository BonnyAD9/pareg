use crate::{reader::Reader, ArgError};

pub struct ParseResult<T> {
    pub err: Option<ArgError>,
    pub res: Option<T>,
}

pub trait FromRead: Sized {
    fn from_read(r: &mut Reader) -> ParseResult<Self>;
}
