use lox_ir::{
    diagnostic::DiagnosticBuilder,
    input_file::InputFile,
    kw::Keyword,
    span::Span,
    syntax::{Expr, Op, Stmt},
    token::Token,
    token_tree::TokenTree,
};

use crate::{
    token_test::{AnyTree, Identifier, Number, StringLiteral, TokenTest},
    tokens::Tokens,
};

pub(crate) struct Parser<'me> {
    db: &'me dyn crate::Db,
    input_file: InputFile,
    tokens: Tokens<'me>,
}

impl<'me> Parser<'me> {
    pub(crate) fn new(db: &'me dyn crate::Db, token_tree: TokenTree) -> Self {
        let tokens = Tokens::new(db, token_tree);
        Self {
            db,
            input_file: token_tree.input_file(db),
            tokens,
        }
    }

    pub(crate) fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = vec![];
        while let Some(stmt) = self.declaration() {
            stmts.push(stmt);
        }
        if self.tokens.peek().is_some() {
            let span = self.tokens.peek_span();
            self.error(span, "extra tokens after statement")
                .emit(self.db);
        }
        stmts
    }

    fn declaration(&mut self) -> Option<Stmt> {
        if self.eat(Keyword::Var).is_some() {
            self.var_declaration()
        } else {
            self.stmt()
        }
    }

    // "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Option<Stmt> {
        if let Some((_, id)) = self.eat(Identifier) {
            let initializer = if self.eat_op(Op::Equal).is_some() {
                let expr = self.parse_expr()?;
                Some(expr)
            } else {
                None
            };
            self.eat(Token::Semicolon)
                .or_report_error(self, || "expected `;`");
            Some(Stmt::VariableDeclaration {
                name: id,
                initializer,
            })
        } else {
            None
        }
    }

    fn stmt(&mut self) -> Option<Stmt> {
        if self.eat(Keyword::Print).is_some() {
            return self.print_stmt();
        } else if let Some((_, token_tree)) = self.delimited('{') {
            // parse a block
            let mut parser = Parser::new(self.db, token_tree);
            let stmts = parser.parse();
            return Some(Stmt::Block(stmts));
        } else if self.eat(Keyword::If).is_some() {
            return self.if_stmt();
        } else if self.eat(Keyword::While).is_some() {
            return self.while_stmt();
        }
        self.expr_stmt()
    }

    fn while_stmt(&mut self) -> Option<Stmt> {
        let (_, token_tree) = self.delimited('(')?;
        let condition = Parser::new(self.db, token_tree).parse_expr()?;
        let body = self.stmt()?;
        Some(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn if_stmt(&mut self) -> Option<Stmt> {
        let (_, token_tree) = self.delimited('(')?;
        let condition = Parser::new(self.db, token_tree).parse_expr()?;
        let then_branch = self.stmt()?;
        let else_branch = if self.eat(Keyword::Else).is_some() {
            self.stmt().map(Box::new)
        } else {
            None
        };
        Some(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    //  "print" expression ";" ;
    fn print_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expr()?;
        self.eat(Token::Semicolon)
            .or_report_error(self, || "expected `;`");
        Some(Stmt::Print(expr))
    }

    fn expr_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expr()?;
        self.eat(Token::Semicolon)
            .or_report_error(self, || "expected `;`");
        Some(Stmt::Expr(expr))
    }

    // expression     -> assignment ;
    // assignment     -> IDENTIFIER "=" assignment | equality ;
    // equality       -> comparison ( ( "!=" | "==" ) comparison )* ;
    // comparison     -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    // term           -> factor ( ( "-" | "+" ) factor )* ;
    // factor         -> unary ( ( "/" | "*" ) unary )* ;
    // unary          -> ( "!" | "-" ) unary
    //             | primary ;
    // primary        → NUMBER | STRING | "true" | "false" | "nil"
    //             | "(" expression ")" ;
    fn parse_expr(&mut self) -> Option<Expr> {
        self.assignment()
    }

    // assignment     -> IDENTIFIER "=" assignment | logic_or ;
    // assignment is not a statement, it is an expression
    fn assignment(&mut self) -> Option<Expr> {
        let expr = self.logic_or()?;
        if self.eat_op(Op::Equal).is_some() {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr {
                return Some(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else {
                // FIXME: we should use the span of the `expr` here
                let span = self.tokens.peek_span();
                self.error(span, "invalid assignment target").emit(self.db);
            }
        }
        Some(expr)
    }

    // logic_or       -> logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> Option<Expr> {
        let mut left = self.logic_and()?;

        loop {
            if self.eat(Keyword::Or).is_some() {
                let right = self.logic_and()?;
                left = Expr::LogicalOr(Box::new(left), Box::new(right));
                continue;
            }
            break;
        }
        Some(left)
    }

    // logic_and       -> equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Option<Expr> {
        let mut left = self.equality()?;

        loop {
            if self.eat(Keyword::And).is_some() {
                let right = self.equality()?;
                left = Expr::LogicalAnd(Box::new(left), Box::new(right));
                continue;
            }
            break;
        }
        Some(left)
    }

    fn equality(&mut self) -> Option<Expr> {
        let mut left = self.comparison()?;

        loop {
            if let Some(right) =
                self.parse_binary(left.clone(), &[Op::NotEqual, Op::EqualEqual], |p| {
                    p.comparison()
                })
            {
                left = right;
                continue;
            }
            break;
        }
        Some(left)
    }

    fn comparison(&mut self) -> Option<Expr> {
        let mut left = self.term()?;

        loop {
            if let Some(right) = self.parse_binary(
                left.clone(),
                &[Op::Greater, Op::GreaterEqual, Op::Less, Op::LessEqual],
                |p| p.term(),
            ) {
                left = right;
                continue;
            }
            break;
        }
        Some(left)
    }

    fn term(&mut self) -> Option<Expr> {
        let mut left = self.factor()?;

        loop {
            if let Some(right) =
                self.parse_binary(left.clone(), &[Op::Minus, Op::Plus], |p| p.factor())
            {
                left = right;
                continue;
            }
            break;
        }
        Some(left)
    }

    fn factor(&mut self) -> Option<Expr> {
        let mut left = self.unary()?;

        loop {
            if let Some(right) =
                self.parse_binary(left.clone(), &[Op::Star, Op::Slash], |p| p.unary())
            {
                left = right;
                continue;
            }
            break;
        }
        Some(left)
    }

    fn parse_binary(
        &mut self,
        left: Expr,
        ops: &[Op],
        parse_rhs: impl Fn(&mut Self) -> Option<Expr>,
    ) -> Option<Expr> {
        for op in ops {
            if self.eat_op(*op).is_some() {
                let right = parse_rhs(self)?;
                let left = Expr::BinaryOp(Box::new(left), *op, Box::new(right));
                return Some(left);
            }
        }
        None
    }

    fn unary(&mut self) -> Option<Expr> {
        for op in &[Op::Minus, Op::Bang] {
            if self.eat_op(*op).is_some() {
                let expr = self.unary()?;
                return Some(Expr::UnaryOp(*op, Box::new(expr)));
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Option<Expr> {
        if self.eat(Keyword::True).is_some() {
            Some(Expr::BooleanLiteral(true))
        } else if self.eat(Keyword::False).is_some() {
            Some(Expr::BooleanLiteral(false))
        } else if self.eat(Keyword::Nil).is_some() {
            Some(Expr::NilLiteral)
        } else if let Some((_, word)) = self.eat(Identifier) {
            Some(Expr::Variable(word))
        } else if let Some((_, word)) = self.eat(Number) {
            Some(Expr::NumberLiteral(word))
        } else if let Some((_, word)) = self.eat(StringLiteral) {
            Some(Expr::StringLiteral(word))
        } else if let Some((_, token_tree)) = self.delimited('(') {
            let expr = Parser::new(self.db, token_tree).parse_expr()?;
            self.eat(Token::Delimiter(')'));
            Some(Expr::Parenthesized(Box::new(expr)))
        } else {
            None
        }
    }

    /// Returns `Some` if the next pending token matches `is`, along
    /// with the narrowed view of the next token.
    fn peek<TT: TokenTest>(&mut self, test: TT) -> Option<TT::Narrow> {
        let span = self.tokens.peek_span().anchor_to(self.input_file);
        test.test(self.db, self.tokens.peek()?, span)
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

    /// If the next token is an opening delimiter, like `(` or `{`,
    /// then consumes it, the token-tree that follows, and the closing delimiter (if present).
    /// Returns the token tree + the span including delimiters.
    /// Reports an error if there is no closing delimiter.
    fn delimited(&mut self, delimiter: char) -> Option<(Span, TokenTree)> {
        let (open_span, _) = self.eat(Token::Delimiter(delimiter))?;

        // Lexer always produces a token tree as the next token after a delimiter:
        let (_, token_tree) = self.eat(AnyTree).unwrap();

        // Consume closing delimiter (if present)
        let closing_delimiter = lox_lex::closing_delimiter(delimiter);
        self.eat(Token::Delimiter(closing_delimiter)).or_else(|| {
            let span = self.tokens.peek_span();
            let message = format!("expected `{}`", closing_delimiter);
            self.error(span, message).emit(self.db);
            None
        });

        let span = open_span.to(self.tokens.last_span());
        Some((span, token_tree))
    }

    fn error(&self, span: Span, message: impl ToString) -> DiagnosticBuilder {
        tracing::debug!("emit error {:?}, {:?}", message.to_string(), span);
        lox_ir::error!(span.anchor_to(self.input_file), "{}", message.to_string())
    }
}

trait OrReportError {
    fn or_report_error<S>(self, parser: &mut Parser<'_>, message: impl FnOnce() -> S) -> Self
    where
        S: ToString;

    fn or_report_error_at<S>(
        self,
        parser: &mut Parser<'_>,
        span: Span,
        message: impl FnOnce() -> S,
    ) -> Self
    where
        S: ToString;
}

impl<T> OrReportError for Option<T> {
    fn or_report_error<S>(self, parser: &mut Parser<'_>, message: impl FnOnce() -> S) -> Self
    where
        S: ToString,
    {
        self.or_report_error_at(parser, parser.tokens.peek_span(), message)
    }

    fn or_report_error_at<S>(
        self,
        parser: &mut Parser<'_>,
        span: Span,
        message: impl FnOnce() -> S,
    ) -> Self
    where
        S: ToString,
    {
        if self.is_some() {
            return self;
        }

        parser.error(span, message()).emit(parser.db);

        None
    }
}
