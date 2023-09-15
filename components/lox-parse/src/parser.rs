use lox_ir::{input_file::InputFile, span::Span, syntax::{Expr, Op}, kw::Keyword, token_tree::TokenTree, token::Token};

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

    pub(crate) fn parse_exprs(&mut self) -> Vec<Expr> {
        let mut exprs = vec![];
        while let Some(expr) = self.parse_expr() {
            exprs.push(expr);
        }
        exprs
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        self.factor()
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;

        loop {
            if let Some(right) = self.parse_binary(left.clone(), &[Op::Star, Op::Slash], |p| p.unary()) {
                left = right;
                continue;
            }
            break;
        }
        Some(left)
    }

    fn parse_binary(&mut self, left: Expr, ops: &[Op], parse_rhs: impl Fn(&mut Self) -> Option<Expr>) -> Option<Expr> {
        for op in ops {
            if let Some(_) = self.eat_op(*op) {
                let right = parse_rhs(self)?;
                let left = Expr::BinaryOp(Box::new(left), *op, Box::new(right));
                return Some(left);
            }
        }
        None
    }

    fn unary(&mut self) -> Option<Expr> {
        for op in &[Op::Minus, Op::Bang] {
            if let Some(_) = self.eat_op(*op) {
                let expr = self.unary()?;
                return Some(Expr::UnaryOp(*op, Box::new(expr)));
            }
        }
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

    /// Peek ahead to see if `op` matches the next set of tokens;
    /// if so, return the span and the tokens after skipping the operator.
    fn test_op(&self, op: Op) -> Option<(Span, Tokens<'me>)> {
        let mut tokens = self.tokens;
        let span0 = tokens.peek_span();

        let mut chars = op.str().chars();

        let ch0 = chars.next().unwrap();
        match tokens.consume() {
            Some(Token::Op(ch1)) if ch0 == ch1 => (),
            _ => return None,
        }

        for ch in chars {
            if tokens.skipped_any() {
                return None;
            }

            match tokens.consume() {
                Some(Token::Op(ch1)) if ch == ch1 => continue,
                _ => return None,
            }
        }

        let span = span0.to(tokens.last_span());

        // Careful: for most operators, if we are looking for `+`
        // and we see `++` or `+-` or something like that,
        // we don't want that to match!

        // If we skipped whitespace, then the token was on its own.
        if tokens.skipped_any() {
            return Some((span, tokens));
        }

        // Only return Some if this is the complete operator
        // (i.e., the operator `=` cannot match against a prefix of the input `==`)
        if let Some(Token::Op(_)) = tokens.peek() {
            return None;
        }

        // ...if not, we've got a match!
        Some((span, tokens))
    }

    /// If the next tokens match the given operator, consume it and
    /// return.
    fn eat_op(&mut self, op: Op) -> Option<Span> {
        let (span, tokens) = self.test_op(op)?;
        self.tokens = tokens;
        Some(span)
    }
}

