pub mod input_file;
pub mod word;


#[salsa::jar(db = Db)]
pub struct Jar(
    word::Word,
    input_file::InputFile,
);

pub trait Db: salsa::DbWithJar<Jar> {}