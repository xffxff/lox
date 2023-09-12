#![feature(trait_upcasting)]

use lex::lex_file;

pub mod lex;


#[salsa::jar(db = Db)]
pub struct Jar(
    lex_file
);

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db {
}