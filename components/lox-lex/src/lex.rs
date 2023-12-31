use std::iter::Peekable;

use lox_ir::{input_file::InputFile, span::Span, token::Token, token_tree::TokenTree, word::Word};

use crate::Db;

#[salsa::tracked]
pub fn lex_file(db: &dyn Db, input_file: InputFile) -> TokenTree {
    let source_text = input_file.source_text(db);
    let chars = &mut source_text.chars().enumerate().peekable();
    let mut lexer = Lexer {
        db,
        input_file,
        chars,
    };
    lexer.lex_tokens(None)
}

pub fn closing_delimiter(ch: char) -> char {
    match ch {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("not a delimiter: {ch:?}"),
    }
}

struct Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    db: &'me dyn Db,
    input_file: InputFile,
    chars: &'me mut Peekable<I>,
}

impl<'me, I> Lexer<'me, I>
where
    I: Iterator<Item = (usize, char)>,
{
    fn lex_tokens(&mut self, end_ch: Option<char>) -> TokenTree {
        let mut tokens = vec![];
        let mut push_token = |t: Token| {
            tracing::debug!("push token: {:?}", t);
            tokens.push(t);
        };

        let mut end_pos = 0;
        let mut start_pos = self.input_file.source_text(self.db).len();
        while let Some((pos, ch)) = self.chars.peek().cloned() {
            end_pos = end_pos.max(pos);
            start_pos = start_pos.min(pos);

            if Some(ch) == end_ch {
                break;
            }

            self.chars.next();
            match ch {
                '(' | '[' | '{' => {
                    push_token(Token::Delimiter(ch));
                    let closing_ch = closing_delimiter(ch);
                    let tree = self.lex_tokens(Some(closing_ch));
                    push_token(Token::Tree(tree));

                    if let Some((_, next_ch)) = self.chars.peek() {
                        if *next_ch == closing_ch {
                            self.chars.next();
                            push_token(Token::Delimiter(closing_ch));
                        }
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let text = self
                        .accumulate(ch, |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'));

                    push_token(Token::Alphabetic(text));
                }
                '0'..='9' => {
                    let text = self.accumulate(ch, |c| c.is_ascii_digit());
                    push_token(Token::Number(text));
                }
                '+' | '-' | '*' | '/' | '!' | '<' | '>' | '=' => {
                    push_token(Token::Op(ch));
                }
                '"' => {
                    let word = self.string_literal();
                    push_token(Token::String(word));
                }
                ' ' => {
                    push_token(Token::Whitespace(ch));
                }
                ';' => {
                    push_token(Token::Semicolon);
                }
                ',' => {
                    push_token(Token::Comma);
                }
                _ => {
                    if ch.is_whitespace() {
                        push_token(Token::Whitespace(ch))
                    } else {
                        push_token(Token::Unknown(ch))
                    }
                }
            }
        }

        TokenTree::new(
            self.db,
            self.input_file,
            Span::from(start_pos, end_pos),
            tokens,
        )
    }

    /// Accumulate `ch0` and following characters while `matches` returns true
    /// into a string.
    fn accumulate_string(&mut self, ch0: char, matches: impl Fn(char) -> bool) -> String {
        let mut string = String::new();
        string.push(ch0);
        while let Some(&(_, ch1)) = self.chars.peek() {
            if !matches(ch1) {
                break;
            }

            string.push(ch1);
            self.chars.next();
        }
        string
    }

    /// Like [`Self::accumulate_string`], but interns the result.
    fn accumulate(&mut self, ch0: char, matches: impl Fn(char) -> bool) -> Word {
        let string = self.accumulate_string(ch0, matches);
        Word::intern(self.db, string)
    }

    // Invoke after consuming the opening quote `"`
    fn string_literal(&mut self) -> Word {
        // FIXME: handle escape sequences
        let mut string = String::new();
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch == '"' {
                self.chars.next();
                break;
            }

            string.push(ch);
            self.chars.next();
        }
        Word::intern(self.db, string)
    }
}

#[cfg(test)]
mod tests {
    use lox_ir::input_file::InputFile;
    use lox_ir::word::Word;

    use crate::Db;
    use crate::Jar;

    use super::lex_file;

    #[salsa::db(Jar, lox_ir::Jar)]
    #[derive(Default)]
    struct Database {
        storage: salsa::Storage<Self>,
    }

    impl salsa::Database for Database {}

    impl lox_ir::Db for Database {}

    impl Db for Database {}

    #[test]
    fn smoke() {
        let db = Database::default();
        let input_file = InputFile::new(&db, Word::intern(&db, "test"), "1 + 2".to_string());
        let token_tree = lex_file(&db, input_file);
        dbg!(token_tree.tokens(&db));
    }
}
