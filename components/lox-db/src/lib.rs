use lox_ir::{input_file::InputFile, word::Word};

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

impl Database {
    pub fn new_input_file(&mut self, name: impl ToString, source_text: String) -> InputFile {
        let name = Word::intern(self, name);
        InputFile::new(self, name, source_text)
    }
}
