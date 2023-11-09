use crate::{token_tree::TokenTree, word::Word};

#[salsa::tracked]
pub struct Function {
    pub name: Word,
    pub params: Vec<Word>,
    pub body: TokenTree,
}
