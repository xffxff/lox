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