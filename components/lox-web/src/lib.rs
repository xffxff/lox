use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn execute(source_text: String) -> String {
    let mut db = lox_db::Database::default();
    let input_file = db.new_input_file("input.lox", source_text);
    lox_execute::execute_file(&db, input_file, None::<fn(_, &lox_execute::VM)>)
}
