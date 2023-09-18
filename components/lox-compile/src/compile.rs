use lox_ir::{input_file::InputFile, bytecode::Chunk};

#[salsa::tracked]
pub fn compile_file(db: &dyn crate::Db, input_file: InputFile) -> Chunk {
    let exprs = lox_parse::parse_file(db, input_file);
    todo!()
}