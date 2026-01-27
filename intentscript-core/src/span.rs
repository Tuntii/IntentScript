use serde::{Deserialize, Serialize};

/// Represents a position in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
    /// Byte offset from start of file (0-indexed)
    pub offset: usize,
}

impl Position {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}

/// Represents a span of source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
    /// Byte offset from start of file (0-indexed)
    pub offset: usize,
    /// Length in bytes
    pub length: usize,
}

impl Span {
    pub fn new(line: u32, column: u32, offset: usize, length: usize) -> Self {
        Self {
            line,
            column,
            offset,
            length,
        }
    }

    pub fn from_position(pos: Position, length: usize) -> Self {
        Self {
            line: pos.line,
            column: pos.column,
            offset: pos.offset,
            length,
        }
    }

    pub fn start_position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
            offset: self.offset,
        }
    }

    pub fn end_offset(&self) -> usize {
        self.offset + self.length
    }

    /// Create a span that covers from the start of self to the end of other
    pub fn to(&self, other: &Span) -> Span {
        let start_offset = self.offset.min(other.offset);
        let end_offset = self.end_offset().max(other.end_offset());
        Span {
            line: self.line,
            column: self.column,
            offset: start_offset,
            length: end_offset - start_offset,
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
            length: 0,
        }
    }
}
