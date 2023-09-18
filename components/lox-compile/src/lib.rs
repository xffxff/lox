#![feature(trait_upcasting)]

pub mod compile;


#[salsa::jar(db = Db)]
pub struct Jar(
    compile::compile_file,
);

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db + lox_parse::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_ir::Db + lox_parse::Db {}