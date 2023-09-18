use lox_compile::compile;
use lox_ir::input_file::InputFile;

use crate::vm::VM;

// FIXME: This should not return a `f64`, but for now it's convenient as our compiler is more or less a calculator for now.
pub fn execute_file(db: &impl crate::Db, input_file: InputFile) -> f64 {
    let chunk = compile::compile_file(db, input_file);
    let mut vm = VM::new(chunk);
    vm.interpret()
}
