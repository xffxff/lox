#![feature(trait_upcasting)]

pub mod compile;
pub use compile::compile_file;
pub use compile::compile_fn;

#[salsa::jar(db = Db)]
pub struct Jar(compile::compile_file, compile::compile_fn);

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db + lox_parse::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_ir::Db + lox_parse::Db {}
