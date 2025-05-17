use std::{
    ffi::OsString,
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};

use crate::{ArgError, FromRead, Reader, Result};

use super::ReadFmt;

/// Simmilar to From read, but can be parsed into an existing instance. This
/// trait is required for parsef. Don't implement this directly, rather
/// implement [`FromRead`] and than use the trait [`AutoSetFromRead`].
pub trait SetFromRead {
    /// Parses data from the reader and sets the current value.
    fn set_from_read<'a>(
        &mut self,
        r: &mut Reader,
        fmt: &'a ReadFmt<'a>,
    ) -> Result<Option<ArgError>>;
}

/// Automatic implementation of SetFromRead for types that support FromRead.
pub trait AutoSetFromRead: FromRead {}

impl<T: AutoSetFromRead> SetFromRead for T {
    fn set_from_read<'a>(
        &mut self,
        r: &mut Reader,
        fmt: &'a ReadFmt<'a>,
    ) -> Result<Option<ArgError>> {
        let err;
        (*self, err) = Self::from_read(r, fmt)?;
        Ok(err)
    }
}

macro_rules! impl_set_from_read {
    ($($t:ident),* $(,)?) => {
        $(impl AutoSetFromRead for $t {})*
    };
}

impl_set_from_read!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    i128,
    f32,
    f64,
    bool,
    char,
    Ipv4Addr,
    SocketAddrV4,
    OsString,
    PathBuf,
);

impl SetFromRead for String {
    fn set_from_read<'a>(
        &mut self,
        r: &mut Reader,
        fmt: &'a ReadFmt<'a>,
    ) -> Result<Option<ArgError>> {
        r.trim_left(fmt)?;
        let (min, max) = fmt.length_range().unwrap_or((0, usize::MAX));
        self.clear();
        r.read_to(self, min)?;
        if self.len() < min {
            return r
                .err_parse(format!(
                    "Expected at least `{min}` characters but there were only \
                `{}` characters.",
                    self.len()
                ))
                .err();
        }
        r.read_to(self, max - min)?;
        if let Some((t, ch)) = fmt.trim() {
            if t.right() {
                let s = if let Some(c) = ch {
                    self[min..].trim_end_matches(c)
                } else {
                    self[min..].trim_ascii_end()
                };
                self.replace_range(min + s.len().., "");
            }
        }

        Ok(Some(r.err_parse(format!(
            "String is too long. Expected at most `{max}` characters."
        ))))
    }
}
