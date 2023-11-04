use wasm_bindgen::prelude::*;
use web_sys::console;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&JsValue::from_str("Hello world!"));

    let compiler = LoxCompiler::default()
        .with_source_text(
            r#"
            var a = 1;
            var b = 2;
            var c = a + b;
            print c;
        "#
            .to_string(),
        )
        .execute();

    console::log_1(&JsValue::from_str(&compiler.output));

    Ok(())
}

#[wasm_bindgen]
pub struct LoxCompiler {
    db: lox_db::Database,

    input_file: lox_ir::input_file::InputFile,

    output: String,
}

impl Default for LoxCompiler {
    fn default() -> Self {
        let mut db = lox_db::Database::default();
        let input_file = db.new_input_file("input.lox", String::new());
        Self {
            db,
            input_file,
            output: String::new(),
        }
    }
}

#[wasm_bindgen]
impl LoxCompiler {
    #[wasm_bindgen]
    pub fn execute(mut self) -> Self {
        let output =
            lox_execute::execute_file(&self.db, self.input_file, None::<fn(_, &lox_execute::VM)>);
        console::log_1(&JsValue::from_str(&output));
        self.output = output;
        self
    }

    #[wasm_bindgen]
    pub fn with_source_text(mut self, source_text: String) -> Self {
        self.input_file
            .set_source_text(&mut self.db)
            .to(source_text);
        self
    }
}
