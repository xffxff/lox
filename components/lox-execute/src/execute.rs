use lox_compile::compile;
use lox_ir::{input_file::InputFile, bytecode};

use crate::vm::VM;

pub fn execute_file(db: &impl crate::Db, input_file: InputFile, step_inspect: Option<impl FnMut(bytecode::Code, &VM)>) {
    let chunk = compile::compile_file(db, input_file);
    let mut vm = VM::new(chunk);
    vm.interpret(step_inspect)
}
