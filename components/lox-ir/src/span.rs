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

    pub fn span_at_start(&self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    pub fn len(&self) -> u32 {
        self.end.0 - self.start.0
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