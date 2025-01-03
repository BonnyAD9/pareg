use crate::{reader::Reader, ArgError};

pub struct ParseResult<T> {
    err: Option<ArgError>,
    res: Option<T>,
    idx: usize,
}

pub trait FromRead: Sized {
    fn from_str2(r: Reader) -> ParseResult<Self>;
}
