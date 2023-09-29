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

    // `foo`
    Variable(Word),

    // assignment expression, like `foo = 1 + 2`
    Assign { name: Word, value: Box<Expr> },

    // logical and
    LogicalAnd(Box<Expr>, Box<Expr>),

    // logical or
    LogicalOr(Box<Expr>, Box<Expr>),
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
            Expr::Variable(word) => write!(f, "Variable({})", word.as_str(db)),
            Expr::Assign { name, value } => f
                .debug_struct("Assign")
                .field("name", &name.as_str(db))
                .field("value", &value.debug(db))
                .finish(),
            Expr::LogicalAnd(left, right) => f
                .debug_struct("LogicalAnd")
                .field("left", &left.debug(db))
                .field("right", &right.debug(db))
                .finish(),
            Expr::LogicalOr(left, right) => f
                .debug_struct("LogicalOr")
                .field("left", &left.debug(db))
                .field("right", &right.debug(db))
                .finish(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    // expression statement, like `1 + 2;`
    Expr(Expr),

    // print statement, like `print 1 + 2;`
    Print(Expr),

    // variable declaration, like `var foo = 1 + 2;`
    VariableDeclaration {
        name: Word,
        initializer: Option<Expr>,
    },

    // block statement, like `{ 1 + 2; }`
    Block(Vec<Stmt>),

    // if statement, like `if (1 + 2) { 3 + 4; } else { 5 + 6; }`
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    // while statement, like `while (1 + 2) { 3 + 4; }`
    While {
        condition: Expr,
        body: Box<Stmt>,
    },

    // for statement, like `for (var i = 0; i < 10; i = i + 1) { print i; }`
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        increment: Option<Expr>,
        body: Box<Stmt>,
    },
}

impl<'db> salsa::DebugWithDb<dyn crate::Db + 'db> for Stmt {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        match self {
            Stmt::Expr(expr) => f
                .debug_struct("Expr")
                .field("expr", &expr.debug(db))
                .finish(),
            Stmt::Print(expr) => f
                .debug_struct("Print")
                .field("expr", &expr.debug(db))
                .finish(),
            Stmt::VariableDeclaration { name, initializer } => f
                .debug_struct("Var")
                .field("name", &name.as_str(db))
                .field("initializer", &initializer.debug(db))
                .finish(),
            Stmt::Block(stmts) => {
                let mut builder = f.debug_struct("Block");
                for stmt in stmts {
                    builder.field("stmt", &stmt.debug(db));
                }
                builder.finish()
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut builder = f.debug_struct("If");
                builder.field("condition", &condition.debug(db));
                builder.field("then_branch", &then_branch.debug(db));
                if let Some(else_branch) = else_branch {
                    builder.field("else_branch", &else_branch.debug(db));
                }
                builder.finish()
            }
            Stmt::While { condition, body } => {
                let mut builder = f.debug_struct("While");
                builder.field("condition", &condition.debug(db));
                builder.field("body", &body.debug(db));
                builder.finish()
            }
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                let mut builder = f.debug_struct("For");
                if let Some(initializer) = initializer {
                    builder.field("initializer", &initializer.debug(db));
                }
                if let Some(condition) = condition {
                    builder.field("condition", &condition.debug(db));
                }
                if let Some(increment) = increment {
                    builder.field("increment", &increment.debug(db));
                }
                builder.field("body", &body.debug(db));
                builder.finish()
            }
        }
    }
}
