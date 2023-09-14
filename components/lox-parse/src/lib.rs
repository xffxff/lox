#![feature(trait_upcasting)]

use file_parser::parse_file;

mod parser;
mod tokens;
mod token_test;
mod tests;
pub mod file_parser;



#[salsa::jar(db = Db)]
pub struct Jar(parse_file);

pub trait Db: salsa::DbWithJar<Jar> + lox_lex::Db + lox_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_lex::Db + lox_ir::Db {}