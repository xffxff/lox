use crate::word::Word;

mod op;
pub use op::Op;


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



impl<'db> salsa::DebugWithDb<dyn crate::Db + 'db> for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, db: &dyn crate::Db, include_all_fields: bool) -> std::fmt::Result {
        match self {
            Expr::NumberLiteral(word) => write!(f, "NumberLiteral({})", word.as_str(db)),
            Expr::UnaryOp(op, expr) => write!(f, "UnaryOp({:?}, {:?})", op, expr.debug(db)),
            _ => todo!()
        }
    }
}