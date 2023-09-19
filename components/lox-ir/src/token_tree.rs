use salsa::DebugWithDb;

use crate::{input_file::InputFile, span::Span, token::Token};

#[salsa::tracked]
pub struct TokenTree {
    pub input_file: InputFile,
    pub span: Span,

    // FIX salsa: if we don't use #[return_ref] here, we get a compile error:
    // expected `Vec<Token>`, found `&Vec<Token>`
    #[return_ref]
    pub tokens: Vec<Token>,
}

impl DebugWithDb<dyn crate::Db> for TokenTree {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        f.debug_struct("TokenTree")
            .field("source text", &self.input_file(db).source_text(db))
            .field("tokens", &self.tokens(db).debug(db))
            .finish()
    }
}
