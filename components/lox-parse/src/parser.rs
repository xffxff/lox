use lox_ir::{input_file::InputFile, span::Span, syntax::Expr, kw::Keyword, token_tree::TokenTree};

use crate::{tokens::Tokens, token_test::{TokenTest, Number}};


pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    input_file: InputFile,
    tokens: Tokens<'me>,
}

impl<'me> Parser<'me> {
    pub(crate) fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let tokens = Tokens::new(db, token_tree);
        Self { db, input_file: token_tree.input_file(db), tokens }
    }

    pub(crate) fn parse_expr(&mut self) -> Option<Expr> {
        self.primary()
    }
    
    fn primary(&mut self) -> Option<Expr> {
        if let Some(_) = self.eat(Keyword::True) {
            Some(Expr::BooleanLiteral(true))
        } else if let Some(_) = self.eat(Keyword::False) {
            Some(Expr::BooleanLiteral(false))
        } else if let Some(_) = self.eat(Keyword::Nil) {
            Some(Expr::NilLiteral)
        } else if let Some((_, word)) = self.eat(Number) {
            Some(Expr::NumberLiteral(word))
        } else {
            None
        }

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

