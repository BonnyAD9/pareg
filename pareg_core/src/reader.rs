use std::{borrow::Cow, io::Read};

use crate::{ArgError, Result};

enum ReaderSource<'a> {
    Io(Box<dyn Read + 'a>),
    Str(Cow<'a, str>),
    Iter(Box<dyn Iterator<Item = char> + 'a>),
    IterErr(Box<dyn Iterator<Item = Result<char>> + 'a>),
}

/// Struct that allows formated reading.
pub struct Reader<'a> {
    source: ReaderSource<'a>,
    peek: Option<char>,
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Read at most `max` chars to the given string.
    pub fn read_to(&mut self, s: &mut String, max: usize) -> Result<()> {
        s.reserve(self.bytes_size_hint().min(max));
        let target = s.len() + max;
        for c in self {
            s.push(c?);
            if s.len() == target {
                break;
            }
        }
        Ok(())
    }

    /// Read all the remaining chars to the given string.
    pub fn read_all(&mut self, s: &mut String) -> Result<()> {
        s.reserve(self.bytes_size_hint());
        for c in self {
            s.push(c?);
        }
        Ok(())
    }

    /// Get the position of the last returned char.
    pub fn pos(&self) -> Option<usize> {
        if self.pos == 0 {
            None
        } else {
            Some(self.pos - 1)
        }
    }

    pub fn bytes_size_hint(&self) -> usize {
        match &self.source {
            ReaderSource::Io(_) => {
                self.peek.map(|a| a.len_utf8()).unwrap_or_default()
            }
            ReaderSource::Str(s) => s.len() - self.pos,
            ReaderSource::Iter(i) => i.size_hint().0,
            ReaderSource::IterErr(i) => i.size_hint().0,
        }
    }

    pub fn map_err(&self, e: ArgError) -> ArgError {
        match &self.source {
            ReaderSource::Str(s) => e
                .shift_span(self.pos.saturating_sub(1), s.to_string())
                .spanned(self.pos.saturating_sub(1)..self.pos),
            _ => e,
        }
    }

    pub fn err_parse(&self, msg: impl Into<Cow<'static, str>>) -> ArgError {
        self.map_err(ArgError::parse_msg(msg, String::new()))
    }

    pub fn err_value(&self, msg: impl Into<Cow<'static, str>>) -> ArgError {
        self.map_err(ArgError::value_msg(msg, String::new()))
    }

    pub fn peek(&mut self) -> Result<Option<char>> {
        if let Some(c) = self.peek {
            Ok(Some(c))
        } else {
            self.peek = self.next().transpose()?;
            Ok(self.peek)
        }
    }

    fn res<T>(&self, res: Result<T>) -> Result<T> {
        res.map_err(|e| self.map_err(e))
    }

    fn new(source: ReaderSource<'a>) -> Self {
        Self {
            source,
            pos: 0,
            peek: None,
        }
    }
}

impl Iterator for Reader<'_> {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.peek {
            self.peek = None;
            return Some(Ok(r));
        }

        let r = match &mut self.source {
            ReaderSource::Io(io) => read_char(io.as_mut()),
            ReaderSource::Str(s) => Ok(s[self.pos..].chars().next()),
            ReaderSource::Iter(i) => Ok(i.next()),
            ReaderSource::IterErr(i) => i.next().transpose(),
        };

        match r {
            Ok(Some(r)) => {
                self.pos += r.len_utf8();
                Some(Ok(r))
            }
            e => self.res(e).transpose(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.source {
            ReaderSource::Io(_) => (self.peek.is_some() as usize, None),
            ReaderSource::Str(s) => (
                self.peek.is_some() as usize + (s.len() - self.pos) / 4,
                Some(self.peek.is_some() as usize + s.len() - self.pos),
            ),
            ReaderSource::Iter(i) => i.size_hint(),
            ReaderSource::IterErr(i) => i.size_hint(),
        }
    }
}

fn read_char<R: Read + ?Sized>(r: &mut R) -> Result<Option<char>> {
    let mut bts = [0; 4];
    if r.read(&mut bts[..1])? != 1 {
        return Ok(None);
    }
    let (len, mut res) = utf8_len(bts[0])?;
    if len == 1 {
        return Ok(Some(res as u8 as char));
    }
    if r.read(&mut bts[1..len])? != len - 1 {
        return Err(ArgError::parse_msg(
            "Utf8 expected more bytes.",
            String::new(),
        ));
    }

    if bts[0] == 0xC0
        || bts[0] == 0xC1
        || (bts[0] == 0xE0 && bts[1] < 0xA0)
        || (bts[0] == 0xF4 && bts[1] < 0x90)
    {
        return Err(ArgError::parse_msg(
            "Utf8 overlong encoding.",
            String::new(),
        ));
    }

    for b in &bts[1..len] {
        if (b & 0xC0) != 0x80 {
            return Err(ArgError::parse_msg(
                "Invalid utf8 trailing byte.",
                String::new(),
            ));
        }
        res = (res << 6) | (b & 0x3F) as u32;
    }

    char::from_u32(res)
        .ok_or_else(|| {
            ArgError::parse_msg("Invalid utf8 code.", String::new())
        })
        .map(Some)
}

fn utf8_len(b: u8) -> Result<(usize, u32)> {
    match b.leading_ones() {
        0 => Ok((1, b as u32)),
        2 => Ok((2, (b & 0x1F) as u32)),
        3 => Ok((3, (b & 0x0F) as u32)),
        4 => Ok((4, (b & 0x07) as u32)),
        _ => Err(ArgError::parse_msg(
            "Invalid leading utf8 byte.",
            String::new(),
        )),
    }
}

impl<'a> From<Box<dyn Read + 'a>> for Reader<'a> {
    fn from(value: Box<dyn Read + 'a>) -> Self {
        Self::new(ReaderSource::Io(value))
    }
}

impl<'a> From<Cow<'a, str>> for Reader<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::new(ReaderSource::Str(value))
    }
}

impl<'a> From<&'a str> for Reader<'a> {
    fn from(value: &'a str) -> Self {
        Cow::Borrowed(value).into()
    }
}

impl From<String> for Reader<'_> {
    fn from(value: String) -> Self {
        Cow::<str>::Owned(value).into()
    }
}

impl<'a> From<Box<dyn Iterator<Item = char> + 'a>> for Reader<'a> {
    fn from(value: Box<dyn Iterator<Item = char> + 'a>) -> Self {
        Self::new(ReaderSource::Iter(value))
    }
}

impl<'a> From<Box<dyn Iterator<Item = Result<char>> + 'a>> for Reader<'a> {
    fn from(value: Box<dyn Iterator<Item = Result<char>> + 'a>) -> Self {
        Self::new(ReaderSource::IterErr(value))
    }
}
