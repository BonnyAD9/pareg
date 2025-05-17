use crate::Reader;
use std::cell::{Ref, RefCell};

use super::{FromRead, ParsedFmt, TrimSide};

/// Format for read function with reader.
#[derive(Debug, Clone, Default)]
pub struct ReadFmt<'a> {
    pub fmt: &'a str,
    parsed: RefCell<Option<ParsedFmt<'a>>>,
}

impl<'a> ReadFmt<'a> {
    /// Create new format from string. The format is parsed lazily.
    pub fn new(fmt: &'a str) -> Self {
        Self {
            fmt,
            parsed: None.into(),
        }
    }

    /// Creates new format where only the base is preserved.
    pub fn keep_base(&self) -> ReadFmt<'_> {
        ReadFmt {
            fmt: "",
            parsed: Some(ParsedFmt {
                base: self.base(),
                ..ParsedFmt::default()
            })
            .into(),
        }
    }

    /// Gets the parsed format.
    pub fn get_parsed(&self) -> ParsedFmt<'a> {
        self.get_parsed_inner().clone()
    }

    /// Get non standart part of the format.
    pub fn custom(&self) -> &'a str {
        self.get_parsed_inner().custom()
    }

    /// Get the trim information from the format.
    pub fn trim(&self) -> Option<(TrimSide, Option<char>)> {
        self.get_parsed_inner().trim()
    }

    /// Get the length range from the format.
    pub fn length_range(&self) -> Option<(usize, usize)> {
        self.get_parsed_inner().length_range()
    }

    /// Get the numerical base from the format.
    pub fn base(&self) -> Option<u32> {
        self.get_parsed_inner().base()
    }

    fn get_parsed_inner(&self) -> Ref<'_, ParsedFmt<'a>> {
        {
            let r = self.parsed.borrow();
            if r.is_some() {
                return Ref::map(r, |o| o.as_ref().unwrap());
            }
        }

        // Parse the input
        {
            *self.parsed.borrow_mut() = Some(self.parse());
        }
        Ref::map(self.parsed.borrow(), |o| o.as_ref().unwrap())
    }

    fn parse(&self) -> ParsedFmt<'a> {
        let mut res = ParsedFmt::default();

        // Check the trimming.
        let mut chrs = self.fmt.char_indices();
        let Some(c1) = chrs.next() else {
            return res;
        };

        let mut fmt = self.fmt;

        if let Some(c2) = chrs.next() {
            fmt = if let Some(t) = TrimSide::from_char(c2.1) {
                res.trim_side = Some(t);
                res.trim_char = Some(c1.1);
                &self.fmt[c2.0 + c2.1.len_utf8()..]
            } else if let Some(t) = TrimSide::from_char(c1.1) {
                res.trim_side = Some(t);
                &self.fmt[c2.0..]
            } else {
                self.fmt
            };
        } else if let Some(t) = TrimSide::from_char(c1.1) {
            res.trim_side = Some(t);
            fmt = &fmt[1..]
        }

        let mut fmt_read: Reader = fmt.into();

        let start = usize::from_read(&mut fmt_read, &"".into())
            .map(|(s, _)| s)
            .ok();

        if start.is_some() {
            fmt = &fmt[fmt_read.pos()..];
        }

        if let Some(f) = fmt.strip_prefix("..") {
            fmt = f;
            let end = usize::from_read(&mut fmt_read, &"".into())
                .map(|(s, _)| s)
                .ok();

            if start.is_some() {
                fmt = &fmt[fmt_read.pos()..];
            }

            res.length_range =
                Some((start.unwrap_or_default(), end.unwrap_or(usize::MAX)));
        } else {
            res.length_range = start.map(|s| (s, s));
        }

        let Some(c) = fmt.chars().next() else {
            res.custom_fmt = fmt;
            return res;
        };

        res.base = match c.to_ascii_lowercase() {
            'd' => Some(10),
            'x' => Some(16),
            'o' => Some(8),
            _ => None,
        };

        if res.base.is_some() {
            fmt = &fmt[1..];
        }

        res.custom_fmt = fmt;
        res
    }
}

impl<'a> From<&'a str> for ReadFmt<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}
