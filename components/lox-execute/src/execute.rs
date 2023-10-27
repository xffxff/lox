use lox_compile::compile;
use lox_ir::{bytecode, input_file::InputFile};

use crate::vm::{ControlFlow, VM};

pub fn execute_file(
    db: &impl crate::Db,
    input_file: InputFile,
    step_inspect: Option<impl FnMut(bytecode::Code, &VM) + Clone>,
) -> String {
    let chunk = compile::compile_file(db, input_file);
    let mut vm = VM::new(chunk);
    loop {
        match vm.step(step_inspect.clone()) {
            ControlFlow::Next => continue,
            ControlFlow::Done => break,
        }
    }
    "hello".to_string()
}
