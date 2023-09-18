#![feature(trait_upcasting)]

pub mod vm;
pub mod execute;

pub use execute::execute_file;


#[salsa::jar(db = Db)]
pub struct Jar();

pub trait Db: salsa::DbWithJar<Jar> + lox_ir::Db + lox_compile::Db {}
impl<T> Db for T where T: salsa::DbWithJar<Jar> + lox_ir::Db + lox_compile::Db {}