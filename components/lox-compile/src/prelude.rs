use crate::compile::compile;
use lox_ir::bytecode::Closure;

pub trait FunctionCompileExt {
    fn compile(&self, db: &dyn crate::Db) -> Closure;
}

impl FunctionCompileExt for lox_ir::function::Function {
    fn compile(&self, db: &dyn crate::Db) -> Closure {
        compile(db, *self)
    }
}
