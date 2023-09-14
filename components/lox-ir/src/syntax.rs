use crate::word::Word;


#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    // 2-character ops
    EqualEqual,
    NotEqual,
    LessEqual,
    GreaterEqual,

    // 1-character ops
    Minus,
    Plus,
    Slash,
    Star,
    Less,
    Greater,
    Not,
}

impl salsa::DebugWithDb<dyn crate::Db> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db, include_all_fields: bool) -> std::fmt::Result {
        match self {
            Expr::NumberLiteral(word) => write!(f, "NumberLiteral({})", word.as_str(db)),
            _ => todo!()
        }
    }
}