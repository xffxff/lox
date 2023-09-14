use crate::parser::Parser;

use lox_ir::{input_file::InputFile, syntax::Expr};

#[salsa::tracked(return_ref)]
pub fn parse_file(db: &dyn crate::Db, input_file: InputFile) -> Option<Expr> {
    let token_tree = lox_lex::lex_file(db, input_file);
    let mut parser = Parser::new(db, token_tree);
    parser.parse_expr()
}
