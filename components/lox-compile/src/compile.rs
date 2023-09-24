use lox_ir::{
    bytecode::{Chunk, Code},
    input_file::InputFile,
    syntax,
};

#[salsa::tracked]
pub fn compile_file(db: &dyn crate::Db, input_file: InputFile) -> Chunk {
    let stmts = lox_parse::parse_file(db, input_file);
    let mut chunk = Chunk::default();
    for stmt in stmts {
        match stmt {
            syntax::Stmt::Expr(expr) => compile_expr(db, expr, &mut chunk),
            syntax::Stmt::Print(expr) => {
                compile_expr(db, expr, &mut chunk);
                chunk.emit_byte(Code::Print)
            },
            syntax::Stmt::Var { name, initializer } => todo!()
        }
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
        syntax::Expr::StringLiteral(word) => {
            let word_str = word.as_str(db);
            let value = word_str.to_string();
            chunk.emit_byte(Code::String(value))
        }
        syntax::Expr::BooleanLiteral(value) => {
            if *value {
                chunk.emit_byte(Code::True)
            } else {
                chunk.emit_byte(Code::False)
            }
        }
        syntax::Expr::NilLiteral => todo!(),
        syntax::Expr::BinaryOp(left, op, right) => {
            compile_expr(db, left, chunk);
            compile_expr(db, right, chunk);
            match op {
                syntax::Op::Plus => chunk.emit_byte(Code::Add),
                syntax::Op::Minus => chunk.emit_byte(Code::Subtract),
                syntax::Op::Slash => chunk.emit_byte(Code::Divide),
                syntax::Op::Star => chunk.emit_byte(Code::Multiply),
                syntax::Op::EqualEqual => chunk.emit_byte(Code::Equal),
                syntax::Op::NotEqual => chunk.emit_byte(Code::NotEqual),
                syntax::Op::Greater => chunk.emit_byte(Code::Greater),
                syntax::Op::GreaterEqual => chunk.emit_byte(Code::GreaterEqual),
                syntax::Op::Less => chunk.emit_byte(Code::Less),
                syntax::Op::LessEqual => chunk.emit_byte(Code::LessEqual),
                _ => todo!(),
            }
        }
        syntax::Expr::UnaryOp(op, expr) => {
            compile_expr(db, expr, chunk);
            match op {
                syntax::Op::Minus => chunk.emit_byte(Code::Negate),
                syntax::Op::Bang => chunk.emit_byte(Code::Not),
                _ => todo!(),
            }
        }
        syntax::Expr::Parenthesized(_) => todo!(),
    }
}
