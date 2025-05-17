use super::TrimSide;

/// Parsed standard format for reader.
#[derive(Debug, Clone, Default)]
pub struct ParsedFmt<'a> {
    pub(super) custom_fmt: &'a str,
    pub(super) length_range: Option<(usize, usize)>,
    pub(super) trim_char: Option<char>,
    pub(super) trim_side: Option<TrimSide>,
    pub(super) base: Option<u32>,
}

impl<'a> ParsedFmt<'a> {
    /// Gets the non standard part of the format.
    pub fn custom(&self) -> &'a str {
        self.custom_fmt
    }

    /// Gets the trim information in the format.
    pub fn trim(&self) -> Option<(TrimSide, Option<char>)> {
        self.trim_side.map(|s| (s, self.trim_char))
    }

    /// Gets the expected length range of the parsed data.
    pub fn length_range(&self) -> Option<(usize, usize)> {
        self.length_range
    }

    /// Gets the base for numerical data.
    pub fn base(&self) -> Option<u32> {
        self.base
    }
}
