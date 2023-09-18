use crate::{token_tree::TokenTree, word::Word};

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

    // Unkown token
    Unknown(char),
}

impl Token {
    pub fn span_len(&self, db: &dyn crate::Db) -> u32 {
        match self {
            Token::Alphabetic(word) | Token::Number(word) => word.as_str(db).len() as u32,
            Token::Op(ch) | Token::Delimiter(ch) | Token::Whitespace(ch) | Token::Unknown(ch) => {
                ch.len_utf8() as u32
            }
            Token::Comment(s) => *s,
            Token::Tree(tree) => tree.span(db).len(),
        }
    }

    pub fn alphabetic(self) -> Option<Word> {
        match self {
            Token::Alphabetic(word) => Some(word),
            _ => None,
        }
    }

    pub fn alphabetic_str(self, db: &dyn crate::Db) -> Option<&str> {
        self.alphabetic().map(|i| i.as_str(db))
    }

    /// Returns `Some` if this is a [`Token::Tree`] variant.
    pub fn tree(self) -> Option<TokenTree> {
        match self {
            Token::Tree(tree) => Some(tree),
            _ => None,
        }
    }
}
