use salsa::DebugWithDb;

use crate::{token_tree::TokenTree, word::Word};

#[salsa::tracked]
pub struct Function {
    pub name: Word,
    pub params: Vec<Word>,
    pub body: TokenTree,
}

impl<'db> DebugWithDb<dyn crate::Db + 'db> for Function {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name(db).as_str(db),
            self.params(db)
                .iter()
                .map(|p| p.as_str(db))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
