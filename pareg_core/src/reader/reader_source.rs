use std::{borrow::Cow, fmt::Debug, io::Read};

use crate::Result;

pub(crate) enum ReaderSource<'a> {
    Io(Box<dyn Read + 'a>),
    Str(Cow<'a, str>),
    Iter(Box<dyn Iterator<Item = char> + 'a>),
    IterErr(Box<dyn Iterator<Item = Result<char>> + 'a>),
}

impl Debug for ReaderSource<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(_) => f.debug_tuple("Io").finish(),
            Self::Str(arg0) => f.debug_tuple("Str").field(arg0).finish(),
            Self::Iter(_) => f.debug_tuple("Iter").finish(),
            Self::IterErr(_) => f.debug_tuple("IterErr").finish(),
        }
    }
}
