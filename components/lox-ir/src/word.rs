use crate::Db;

#[salsa::interned]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Word {
    #[return_ref]
    pub string: String,
}

impl Word {
    pub fn intern(db: &dyn Db, string: impl ToString) -> Word {
        Word::new(db, string.to_string())
    }

    pub fn as_str(self, db: &dyn Db) -> &str {
        self.string(db)
    }
}
