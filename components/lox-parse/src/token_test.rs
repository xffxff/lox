use lox_ir::{token::Token, span::FileSpan};
use std::fmt::Debug;



/// Represents some kind of "condition test" that can be applied to a single token
/// (e.g., is an identifier or is a keyword).
pub(crate) trait TokenTest: std::fmt::Debug {
    /// When the test is successful, we return the token back but (potentially)
    /// with a narrower, more specific type -- this is that type.
    type Narrow: Debug;

    /// If `token` matches the condition, return `Some` with a potentially transformed
    /// version of the token. Else returns None.
    fn test(self, db: &dyn crate::Db, token: Token, span: FileSpan) -> Option<Self::Narrow>;
}