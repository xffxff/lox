use crate::input_file::InputFile;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Offset,
    pub end: Offset,
}

impl Span {
    pub fn from(start: impl Into<Offset>, end: impl Into<Offset>) -> Self {
        let this = Self {
            start: start.into(),
            end: end.into(),
        };
        assert!(this.start <= this.end);
        this
    }

    pub fn anchor_to(self, anchored: InputFile) -> FileSpan {
        FileSpan {
            input_file: anchored,
            start: self.start,
            end: self.end,
        }
    }

    pub fn span_at_start(&self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    pub fn len(&self) -> u32 {
        self.end.0 - self.start.0
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn to(self, other: Span) -> Span {
        assert!(self.start <= other.start && other.end >= self.end);
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 0-based byte offset within a file.
pub struct Offset(u32);

impl std::ops::Add<u32> for Offset {
    type Output = Offset;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl From<usize> for Offset {
    fn from(value: usize) -> Offset {
        assert!(value < std::u32::MAX as usize);
        Offset(value as u32)
    }
}

impl From<u32> for Offset {
    fn from(value: u32) -> Offset {
        Offset(value)
    }
}

impl From<Offset> for u32 {
    fn from(offset: Offset) -> Self {
        offset.0
    }
}

impl From<Offset> for usize {
    fn from(offset: Offset) -> Self {
        offset.0 as usize
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileSpan {
    pub input_file: InputFile,
    pub start: Offset,
    pub end: Offset,
}

impl FileSpan {
    pub fn snippet<'db>(&self, db: &'db dyn crate::Db) -> &'db str {
        &self.input_file.source_text(db)[usize::from(self.start)..usize::from(self.end)]
    }

    /// True if the given character falls within this span.
    pub fn contains(&self, offset: Offset) -> bool {
        self.start <= offset && offset < self.end
    }
}

impl std::fmt::Debug for FileSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}..@{}", self.start.0, self.end.0)
    }
}
