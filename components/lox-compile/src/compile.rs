use lox_ir::{
    bytecode::{Chunk, Code},
    input_file::InputFile,
    syntax,
};

#[salsa::tracked]
pub fn compile_file(db: &dyn crate::Db, input_file: InputFile) -> Chunk {
    let exprs = lox_parse::parse_file(db, input_file);
    let mut chunk = Chunk::default();
    for expr in exprs {
        compile_expr(db, expr, &mut chunk);
    }
    chunk
}

fn compile_expr(db: &dyn crate::Db, expr: &syntax::Expr, chunk: &mut Chunk) {
    match expr {
        syntax::Expr::NumberLiteral(word) => {
            let word_str = word.as_str(db);
            let value = word_str.parse::<f64>().unwrap();
            chunk.emit_byte(Code::Constant(value.into()))
        }
        syntax::Expr::StringLiteral(_) => todo!(),
        syntax::Expr::BooleanLiteral(value) => {
            if *value {
                chunk.emit_byte(Code::True)
            } else {
                chunk.emit_byte(Code::False)
            }
        },
        syntax::Expr::NilLiteral => todo!(),
        syntax::Expr::BinaryOp(left, op, right) => {
            compile_expr(db, left, chunk);
            compile_expr(db, right, chunk);
            match op {
                syntax::Op::Plus => chunk.emit_byte(Code::Add),
                syntax::Op::Minus => chunk.emit_byte(Code::Subtract),
                syntax::Op::Slash => chunk.emit_byte(Code::Divide),
                syntax::Op::Star => chunk.emit_byte(Code::Multiply),
                _ => todo!(),
            }
        }
        syntax::Expr::UnaryOp(_, _) => todo!(),
        syntax::Expr::Parenthesized(_) => todo!(),
    }
}
