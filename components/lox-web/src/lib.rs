use lox_ir::diagnostic::Diagnostics;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn execute(source_text: String) -> String {
    let mut db = lox_db::Database::default();
    let input_file = db.new_input_file("input.lox", source_text);
    lox_compile::compile_file(&db, input_file);
    let diagnostics = lox_compile::compile_file::accumulated::<Diagnostics>(&db, input_file);
    if !diagnostics.is_empty() {
        lox_error_format::format_diagnostics(&db, &diagnostics).unwrap()
    } else {
        lox_execute::execute_file(&db, input_file, None::<fn(_, &lox_execute::VM)>)
    }
}
