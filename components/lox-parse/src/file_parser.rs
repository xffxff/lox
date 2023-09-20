use crate::parser::Parser;

use lox_ir::{input_file::InputFile, syntax::Stmt};

#[salsa::tracked(return_ref)]
pub fn parse_file(db: &dyn crate::Db, input_file: InputFile) -> Vec<Stmt> {
    let token_tree = lox_lex::lex_file(db, input_file);
    let mut parser = Parser::new(db, token_tree);
    parser.parse()
}
