use crate::Result;

use super::{Reader, ReaderSource};


/// Char iterator over reader.
pub struct ReaderChars<'r, 'a>(pub(crate) &'r mut Reader<'a>);


impl Iterator for ReaderChars<'_, '_> {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0.source {
            ReaderSource::Io(_) => (self.0.peek.is_some() as usize, None),
            ReaderSource::Str(s) => (
                self.0.peek.is_some() as usize + (s.len() - self.0.pos) / 4,
                Some(self.0.peek.is_some() as usize + s.len() - self.0.pos),
            ),
            ReaderSource::Iter(i) => i.size_hint(),
            ReaderSource::IterErr(i) => i.size_hint(),
        }
    }
}
