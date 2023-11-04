#[salsa::db(
    lox_parse::Jar,
    lox_ir::Jar,
    lox_lex::Jar,
    lox_compile::Jar,
    lox_execute::Jar,
    lox_error_format::Jar
)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

impl lox_ir::Db for Database {}

impl lox_lex::Db for Database {}
