use crate::word::Word;

#[salsa::input]
pub struct InputFile {
    pub name: Word,

    // the raw contents of the input file, as a string
    #[return_ref]
    pub source_text: String,
}
