use std::{borrow::Cow, io::Read};

use reader_source::ReaderSource;

use crate::{ArgError, Result};

mod from_read;
mod parsed_fmt;
mod read_fmt;
mod reader_chars;
mod reader_source;
mod set_from_read;
mod trim_side;

pub use self::{
    from_read::*, parsed_fmt::*, read_fmt::*, reader_chars::*,
    set_from_read::*, trim_side::*,
};

/// Struct that allows formated reading.
pub struct Reader<'a> {
    source: ReaderSource<'a>,
    undone: Vec<char>,
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Read at most `max` chars to the given string.
    pub fn read_to(&mut self, s: &mut String, max: usize) -> Result<()> {
        s.reserve(self.bytes_size_hint().min(max));
        let target = s.len().saturating_add(max);
        for c in self.chars() {
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
        for c in self.chars() {
            s.push(c?);
        }
        Ok(())
    }

    /// Get the position of the last returned char.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Gets the low estimate of the remaining bytes.
    pub fn bytes_size_hint(&self) -> usize {
        match &self.source {
            ReaderSource::Io(_) => self.undone.len(),
            ReaderSource::Str(s) => s.len() - self.pos + self.undone.len(),
            ReaderSource::Iter(i) => i.size_hint().0 + self.undone.len(),
            ReaderSource::IterErr(i) => i.size_hint().0 + self.undone.len(),
        }
    }

    /// Adds relevant information to the given error.
    pub fn map_err(&self, e: ArgError) -> ArgError {
        match &self.source {
            ReaderSource::Str(s) => e
                .shift_span(self.pos.saturating_sub(1), s.to_string())
                .spanned(self.pos.saturating_sub(1)..self.pos),
            _ => e,
        }
    }

    /// Creates parse error with the given message.
    pub fn err_parse(&self, msg: impl Into<Cow<'static, str>>) -> ArgError {
        self.map_err(ArgError::parse_msg(msg, String::new()))
    }

    /// Creates value error with the given message.
    pub fn err_value(&self, msg: impl Into<Cow<'static, str>>) -> ArgError {
        self.map_err(ArgError::value_msg(msg, String::new()))
    }

    /// Adds relevant information to the given error. The span will start
    /// at the next character.
    pub fn map_err_peek(&self, e: ArgError) -> ArgError {
        match &self.source {
            ReaderSource::Str(s) => e
                .shift_span(self.pos, s.to_string())
                .spanned(self.pos..self.pos),
            _ => e,
        }
    }

    /// Creates parse error with the given message. Span will start at the next
    /// character.
    pub fn err_parse_peek(
        &self,
        msg: impl Into<Cow<'static, str>>,
    ) -> ArgError {
        self.map_err_peek(ArgError::parse_msg(msg, String::new()))
    }

    /// Creates value error with the given message. Span will start at the next
    /// character.
    pub fn err_value_peek(
        &self,
        msg: impl Into<Cow<'static, str>>,
    ) -> ArgError {
        self.map_err(ArgError::value_msg(msg, String::new()))
    }

    /// Peek at the next character.
    pub fn peek(&mut self) -> Result<Option<char>> {
        if self.undone.is_empty() {
            let n = self.next_inner()?;
            self.undone.extend(n);
        }
        Ok(self.undone.last().copied())
    }

    /// Match the given string to the output. If it doesn't match, return
    /// error.
    pub fn expect(&mut self, s: &str) -> Result<()> {
        for p in s.chars() {
            let Some(s) = self.next()? else {
                return self
                    .err_parse("Unexpected end of string.")
                    .inline_msg(format!("Expected `{p}` to form `{s}`"))
                    .err();
            };
            if p != s {
                return self
                    .err_parse(format!("Unexpected character `{s}`."))
                    .inline_msg(format!("Expected `{p}` to form `{s}`."))
                    .err();
            }
        }
        Ok(())
    }

    /// Skips characters while the given function matches.
    pub fn skip_while(
        &mut self,
        mut f: impl FnMut(char) -> bool,
    ) -> Result<()> {
        while let Some(c) = self.peek()? {
            if !f(c) {
                break;
            }
            self.next()?;
        }
        Ok(())
    }

    /// Checks if the next char is the given char. If yes, returns true and
    /// moves to the next position.
    pub fn is_next_some(&mut self, c: char) -> Result<bool> {
        if matches!(self.peek()?, Some(v) if v == c) {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Checks if the next value matches the predicate. If yes, returns true
    /// and moves to the next position.
    pub fn is_next(
        &mut self,
        p: impl FnOnce(Option<char>) -> bool,
    ) -> Result<bool> {
        if p(self.peek()?) {
            self.next()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Gets the next character.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Option<char>> {
        let c = if let Some(c) = self.undone.pop() {
            c
        } else if let Some(c) = self.next_inner()? {
            c
        } else {
            return Ok(None);
        };

        self.pos += c.len_utf8();
        Ok(Some(c))
    }

    /// Gets iterator over chars.
    pub fn chars(&mut self) -> ReaderChars<'_, 'a> {
        ReaderChars(self)
    }

    /// Parses the next value.
    pub fn parse<'f, T: FromRead>(
        &mut self,
        fmt: &'f ReadFmt<'f>,
    ) -> Result<(T, Option<ArgError>)> {
        T::from_read(self, fmt)
    }

    /// Trims characters from the left side according to the given format.
    pub fn trim_left(&mut self, fmt: &ReadFmt) -> Result<()> {
        let Some((t, chr)) = fmt.trim() else {
            return Ok(());
        };

        if !t.left() {
            return Ok(());
        }

        if let Some(c) = chr {
            self.skip_while(|a| a == c)
        } else {
            self.skip_while(|a| a.is_ascii_whitespace())
        }
    }

    /// Trims characters from the right side according to the given format.
    pub fn trim_right(&mut self, fmt: &ReadFmt) -> Result<()> {
        let Some((t, chr)) = fmt.trim() else {
            return Ok(());
        };

        if !t.right() {
            return Ok(());
        }

        if let Some(c) = chr {
            self.skip_while(|a| a == c)
        } else {
            self.skip_while(|a| a.is_ascii_whitespace())
        }
    }

    /// Prepends the given character to the reader.
    pub fn unnext(&mut self, c: char) {
        self.pos = self.pos.saturating_sub(c.len_utf8());
        self.undone.push(c);
    }

    /// Prepends the given characters to the reader.
    pub fn prepend<I: IntoIterator<Item = char>>(&mut self, s: I)
    where
        I::IntoIter: DoubleEndedIterator,
    {
        for c in s.into_iter().rev() {
            self.unnext(c);
        }
    }

    fn next_inner(&mut self) -> Result<Option<char>> {
        let r = match &mut self.source {
            ReaderSource::Io(io) => read_char(io.as_mut()),
            ReaderSource::Str(s) => Ok(s[self.pos..].chars().next()),
            ReaderSource::Iter(i) => Ok(i.next()),
            ReaderSource::IterErr(i) => i.next().transpose(),
        };
        self.res(r)
    }

    fn res<T>(&self, res: Result<T>) -> Result<T> {
        res.map_err(|e| self.map_err(e))
    }

    fn new(source: ReaderSource<'a>) -> Self {
        Self {
            source,
            pos: 0,
            undone: vec![],
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
