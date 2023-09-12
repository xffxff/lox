use crate::word::Word;



#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    // "foo", could be keyword or identifier
    Alphabetic(Word),

    // "22_000"
    Number(Word),

    // A single character from an operator, like "+"
    Op(char),

    // `(`, `)`, `[`, `]`, `{`, `}`
    Delimiter(char),

    /// Some whitespace (` `, `\n`, etc)
    Whitespace(char),
}