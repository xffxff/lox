use lox_ir::{kw::Keyword, span::FileSpan, token::Token, token_tree::TokenTree, word::Word};
use std::fmt::Debug;

/// Represents some kind of "condition test" that can be applied to a single token
/// (e.g., is an identifier or is a keyword).
pub(crate) trait TokenTest: std::fmt::Debug {
    /// When the test is successful, we return the token back but (potentially)
    /// with a narrower, more specific type -- this is that type.
    type Narrow: Debug;

    /// If `token` matches the condition, return `Some` with a potentially transformed
    /// version of the token. Else returns None.
    fn test(self, db: &dyn crate::Db, token: Token, span: FileSpan) -> Option<Self::Narrow>;
}

impl TokenTest for Keyword {
    type Narrow = Self;

    fn test(self, db: &dyn crate::Db, token: Token, _span: FileSpan) -> Option<Self> {
        let Some(str) = token.alphabetic_str(db) else {
            return None;
        };

        if str == self.str() {
            Some(self)
        } else {
            None
        }
    }
}

/// A number like `22`.
///
/// Note that `.` is not accepted.
/// Floating point literals can be parsed by combining multiple tokens.
#[derive(Debug)]
pub(crate) struct Number;
impl TokenTest for Number {
    type Narrow = Word;

    fn test(self, _db: &dyn crate::Db, token: Token, _span: FileSpan) -> Option<Word> {
        match token {
            Token::Number(w) => Some(w),
            _ => None,
        }
    }
}

impl TokenTest for Token {
    type Narrow = Token;

    fn test(self, _: &dyn crate::Db, token: Token, _span: FileSpan) -> Option<Token> {
        if self == token {
            Some(token)
        } else {
            None
        }
    }
}

/// Any token at all
#[derive(Debug)]
pub(crate) struct AnyTree;
impl TokenTest for AnyTree {
    type Narrow = TokenTree;

    fn test(self, _: &dyn crate::Db, token: Token, _span: FileSpan) -> Option<TokenTree> {
        token.tree()
    }
}

#[cfg(test)]
mod tests {
    use lox_ir::{input_file::InputFile, kw::Keyword, span::FileSpan, token::Token, word::Word};

    use crate::token_test::TokenTest;

    #[salsa::db(crate::Jar, lox_ir::Jar, lox_lex::Jar)]
    #[derive(Default)]
    struct Database {
        storage: salsa::Storage<Self>,
    }

    impl salsa::Database for Database {}

    impl lox_ir::Db for Database {}

    impl lox_lex::Db for Database {}

    fn fake_file_span(db: &dyn crate::Db) -> FileSpan {
        let fake_file = InputFile::new(db, Word::intern(db, "foo"), "foo".to_string());
        FileSpan {
            input_file: fake_file,
            start: 0u32.into(),
            end: 3u32.into(),
        }
    }

    #[test]
    fn token_test_for_keyword() {
        let db = &mut Database::default();
        let file_span = fake_file_span(db);
        let test_keyword = |keyword: Keyword, token_str: &str| {
            let token = Token::Alphabetic(Word::intern(db, token_str));
            assert_eq!(keyword.test(db, token, file_span), Some(keyword));
        };

        test_keyword(Keyword::True, "true");
        test_keyword(Keyword::False, "false");
        test_keyword(Keyword::Nil, "nil");

        let token = Token::Alphabetic(Word::intern(db, "foo"));
        assert_eq!(Keyword::True.test(db, token, file_span), None);
    }
}
