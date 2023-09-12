use crate::word::Word;


#[salsa::input]
pub struct InputFile {
    name: Word,

    // the raw contents of the input file, as a string.
    #[return_ref]
    source_text: String
}