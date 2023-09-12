#![feature(trait_upcasting)]

mod parser;
mod tokens;



#[salsa::jar(db = Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + lox_lex::Db + lox_ir::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_lex::Db + lox_ir::Db {}