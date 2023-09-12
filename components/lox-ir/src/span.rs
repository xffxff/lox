#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Offset,
    pub end: Offset,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// 0-based byte offset within a file.
pub struct Offset(u32);