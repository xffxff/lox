use crate::{word::Word, token_tree::TokenTree};



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

    // Some whitespace (` `, `\n`, etc)
    Whitespace(char),

    // When we encounter an opening delimiter, all the contents up to (but not including)
    // the closing delimiter are read into a Tree.
    Tree(TokenTree),
}