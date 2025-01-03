use crate::Result;

pub struct Reader {
    // TODO
}

impl Reader {
    pub fn read_to(&mut self, s: &mut String, max: usize) -> Result<()> {
        s.reserve(self.size_hint().0.min(max));
        let target = s.len() + max;
        for c in self {
            s.push(c?);
            if s.len() == target {
                break;
            }
        }
        Ok(())
    }

    pub fn read_all(&mut self, s: &mut String) -> Result<()> {
        s.reserve(self.size_hint().0);
        for c in self {
            s.push(c?);
        }
        Ok(())
    }
}

impl Iterator for Reader {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO
        todo!()
    }
}
