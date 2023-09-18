#![feature(trait_upcasting)]

pub mod lex;

pub use lex::closing_delimiter;
pub use lex::lex_file;

#[salsa::jar(db = Db)]
pub struct Jar(lex_file);

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db {}
