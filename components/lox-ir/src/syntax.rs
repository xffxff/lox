use crate::word::Word;

mod op;
pub use op::Op;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    // `22`
    NumberLiteral(Word),

    // `"foo"`
    StringLiteral(Word),

    // true, false
    BooleanLiteral(bool),

    // nil
    NilLiteral,

    // binary expression
    BinaryOp(Box<Expr>, Op, Box<Expr>),

    // unary expression
    UnaryOp(Op, Box<Expr>),

    // `(expr)`
    Parenthesized(Box<Expr>),
}

impl<'db> salsa::DebugWithDb<dyn crate::Db + 'db> for Expr {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        match self {
            Expr::NumberLiteral(word) => write!(f, "NumberLiteral({})", word.as_str(db)),
            Expr::UnaryOp(op, expr) => f
                .debug_struct("UnaryOp")
                .field("op", op)
                .field("expr", &expr.debug(db))
                .finish(),
            Expr::BinaryOp(left, op, right) => f
                .debug_struct("BinaryOp")
                .field("left", &left.debug(db))
                .field("op", op)
                .field("right", &right.debug(db))
                .finish(),
            Expr::Parenthesized(expr) => f
                .debug_struct("Parenthesized")
                .field("expr", &expr.debug(db))
                .finish(),
            Expr::BooleanLiteral(value) => write!(f, "BooleanLiteral({})", value),
            Expr::StringLiteral(word) => write!(f, "StringLiteral({})", word.as_str(db)),
            _ => todo!(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    // expression statement, like `1 + 2;`
    Expr(Expr),
}

impl salsa::DebugWithDb<dyn crate::Db> for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db, _include_all_fields: bool) -> std::fmt::Result {
        match self {
            Stmt::Expr(expr) => f.debug_struct("Expr").field("expr", &expr.debug(db)).finish(),
        }
    }
}