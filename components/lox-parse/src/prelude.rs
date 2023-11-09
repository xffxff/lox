use lox_ir::syntax::Stmt;

use crate::parser::Parser;

pub trait FunctionParseExt {
    fn parse(&self, db: &dyn crate::Db) -> Vec<Stmt>;
}

impl FunctionParseExt for lox_ir::function::Function {
    fn parse(&self, db: &dyn crate::Db) -> Vec<Stmt> {
        let mut parser = Parser::new(db, self.body(db));
        parser.parse()
    }
}
