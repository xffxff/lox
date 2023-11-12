use lox_ir::{bytecode, input_file::InputFile};

use crate::{
    kernel::Kernel,
    vm::{ControlFlow, VM},
};

#[salsa::tracked]
pub fn main_function(db: &dyn crate::Db, input_file: InputFile) -> lox_ir::function::Function {
    let tree = lox_lex::lex_file(db, input_file);
    let name = lox_ir::word::Word::new(db, "main".to_string());
    lox_ir::function::Function::new(db, name, vec![], tree)
}

pub fn execute_file(
    db: &impl crate::Db,
    input_file: InputFile,
    kernel: &mut impl Kernel,
    diagnostic_with_color: bool,
    step_inspect: Option<impl FnMut(Option<bytecode::Code>, &VM) + Clone>,
) {
    let main = main_function(db, input_file);
    let mut vm = VM::new(db, main, diagnostic_with_color);

    while let ControlFlow::Next = vm.step(db, kernel, step_inspect.clone()) {}
}
