/// Determines side from which should be trimmed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TrimSide {
    /// Trim from start.
    Left,
    /// Trim from end.
    Right,
    /// Trim on both sides.
    Both,
}

impl TrimSide {
    /// Get the trim side from the representing character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '<' => Some(Self::Right),
            '^' => Some(Self::Both),
            '>' => Some(Self::Left),
            _ => None,
        }
    }

    /// Checks whether should trim on left.
    pub fn left(&self) -> bool {
        matches!(self, Self::Left | Self::Both)
    }

    /// Checks whether should trim on right.
    pub fn right(&self) -> bool {
        matches!(self, Self::Right | Self::Both)
    }
}
