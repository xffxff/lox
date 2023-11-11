#![feature(trait_upcasting)]

pub mod execute;
pub mod kernel;
pub mod vm;

pub use execute::execute_file;
pub use vm::VM;

#[salsa::jar(db = Db)]
pub struct Jar(execute::main_function);

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db + lox_compile::Db + lox_error_format::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_ir::Db + lox_compile::Db + lox_error_format::Db
{}
