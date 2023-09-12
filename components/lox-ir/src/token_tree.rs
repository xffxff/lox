use crate::{input_file::InputFile, span::Span, token::Token};


#[salsa::tracked]
pub struct TokenTree {
    input_file: InputFile,
    span: Span,

    // FIX salsa: if we don't use #[return_ref] here, we get a compile error:
    // expected `Vec<Token>`, found `&Vec<Token>`
    #[return_ref]
    pub tokens: Vec<Token>,
}