use lox_compile::compile;
use lox_ir::input_file::InputFile;

use crate::vm::{VM, Value};

pub fn execute_file(db: &impl crate::Db, input_file: InputFile) -> Value {
    let chunk = compile::compile_file(db, input_file);
    let mut vm = VM::new(chunk);
    vm.interpret()
}
