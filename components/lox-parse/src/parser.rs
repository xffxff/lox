use lox_ir::{input_file::InputFile, span::Span};

use crate::{tokens::Tokens, token_test::TokenTest};


pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    input_file: InputFile,
    tokens: Tokens<'me>,
}

impl<'me> Parser<'me> {
    
    fn primary(&mut self) {
        todo!()
    }

    /// Returns `Some` if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek<TT: TokenTest>(&mut self, test: TT) -> Option<TT::Narrow> {
        let span = self.tokens.peek_span().anchor_to(self.db, self.input_file);
        test.test(self.db, self.tokens.peek()?, span)
    }

    /// Span of the next pending token, or the span of EOF if there is no next token.
    fn peek_span(&mut self) -> Span {
        self.tokens.peek_span()
    }

    /// If the next pending token matches `test`, consumes it and
    /// returns the span + narrowed view. Otherwise returns None
    /// and has no effect. Returns None if there is no pending token.
    fn eat<TT: TokenTest>(&mut self, test: TT) -> Option<(Span, TT::Narrow)> {
        let span = self.tokens.peek_span();
        let narrow = self.peek(test)?;
        self.tokens.consume();
        Some((span, narrow))
    }
}

