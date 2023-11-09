use lox_ir::bytecode::CompiledFunction;

use crate::compile::compile;

pub trait FunctionCompileExt {
    fn compile(&self, db: &dyn crate::Db) -> CompiledFunction;
}

impl FunctionCompileExt for lox_ir::function::Function {
    fn compile(&self, db: &dyn crate::Db) -> CompiledFunction {
        compile(db, *self)
    }
}
