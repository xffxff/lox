use crate::{word::Word, token_tree::TokenTree};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// `# ...`, argument is the length (including `#`).
    /// Note that the newline that comes after a comment is
    /// considered a separate whitespace token.
    Comment(u32),

    // When we encounter an opening delimiter, all the contents up to (but not including)
    // the closing delimiter are read into a Tree.
    Tree(TokenTree),
}

impl Token {
    pub fn span_len(&self, db: &dyn crate::Db) -> u32 {
        match self {
            Token::Alphabetic(word) | Token::Number(word) => word.as_str(db).len() as u32,
            Token::Op(ch) | Token::Delimiter(ch) | Token::Whitespace(ch) => ch.len_utf8() as u32,
            Token::Comment(s) => *s,
            Token::Tree(tree) => tree.span(db).len(),
        }
    }
}