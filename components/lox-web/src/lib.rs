use lox_execute::kernel::BufferKernel;
use lox_ir::{diagnostic::Diagnostics, input_file::InputFile};
use salsa::DebugWithDb;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Compiler {
    db: lox_db::Database,

    input_file: InputFile,
}

impl Default for Compiler {
    fn default() -> Self {
        let db = lox_db::Database::default();
        let input_file = db.new_input_file("input.lox", String::new());
        Self { db, input_file }
    }
}

#[wasm_bindgen]
pub fn compiler() -> Compiler {
    Compiler::default()
}

#[wasm_bindgen]
impl Compiler {
    pub fn set_source_text(&mut self, source_text: String) {
        self.input_file
            .set_source_text(&mut self.db)
            .to(source_text);
    }

    pub fn execute(&mut self) -> String {
        lox_compile::compile_file(&self.db, self.input_file);
        let diagnostics =
            lox_compile::compile_file::accumulated::<Diagnostics>(&self.db, self.input_file);
        if !diagnostics.is_empty() {
            lox_error_format::format_diagnostics(&self.db, &diagnostics).unwrap()
        } else {
            let mut kernel = BufferKernel::new();
            lox_execute::execute_file(
                &self.db,
                self.input_file,
                &mut kernel,
                true,
                None::<fn(_, &lox_execute::VM)>,
            );
            kernel.take_buffer()
        }
    }

    pub fn parse(&mut self) -> String {
        let exprs = lox_parse::parse_file(&self.db, self.input_file);

        let mut buf = String::new();
        for expr in exprs.iter() {
            buf.push_str(&format!("{:#?}\n", expr.debug(&self.db)));
        }
        buf
    }

    pub fn bytecode(&mut self) -> String {
        let chunk = lox_compile::compile_file(&self.db, self.input_file);
        format!("{:#?}", chunk)
    }
}
