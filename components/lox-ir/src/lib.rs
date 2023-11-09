pub mod bytecode;
pub mod diagnostic;
pub mod function;
pub mod input_file;
pub mod kw;
pub mod span;
pub mod syntax;
pub mod token;
pub mod token_tree;
pub mod word;

#[salsa::jar(db = Db)]
pub struct Jar(
    word::Word,
    input_file::InputFile,
    token_tree::TokenTree,
    diagnostic::Diagnostics,
    kw::Keywords,
    kw::keywords_map,
    function::Function,
);

pub trait Db: salsa::DbWithJar<Jar> {}
