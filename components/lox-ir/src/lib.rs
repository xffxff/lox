pub mod input_file;
pub mod word;
pub mod span;
pub mod token;
pub mod token_tree;
pub mod syntax;
pub mod kw;
pub mod diagnostic;


#[salsa::jar(db = Db)]
pub struct Jar(
    word::Word,
    input_file::InputFile,
    token_tree::TokenTree,
    diagnostic::Diagnostics,
);

pub trait Db: salsa::DbWithJar<Jar> {}