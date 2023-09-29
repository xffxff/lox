use lox_ir::{
    bytecode::{Chunk, Code},
    input_file::InputFile,
    syntax,
};

#[salsa::tracked]
pub fn compile_file(db: &dyn crate::Db, input_file: InputFile) -> Chunk {
    let stmts = lox_parse::parse_file(db, input_file);
    let mut chunk = Chunk::default();
    let mut compiler = Compiler::default();
    for stmt in stmts {
        compiler.compile_stmt(db, stmt, &mut chunk);
    }
    chunk
}

struct Local {
    name: String,
    depth: usize,
}

impl Local {
    fn new(name: &str, depth: usize) -> Self {
        Self {
            name: name.to_string(),
            depth,
        }
    }
}

#[derive(Default)]
struct Compiler {
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    fn compile_stmt(&mut self, db: &dyn crate::Db, stmt: &syntax::Stmt, chunk: &mut Chunk) {
        match stmt {
            syntax::Stmt::Expr(expr) => {
                self.compile_expr(db, expr, chunk);
                chunk.emit_byte(Code::Pop);
            }
            syntax::Stmt::Print(expr) => {
                self.compile_expr(db, expr, chunk);
                chunk.emit_byte(Code::Print);
            }
            syntax::Stmt::VariableDeclaration { name, initializer } => {
                if let Some(initializer) = initializer {
                    self.compile_expr(db, initializer, chunk);
                } else {
                    chunk.emit_byte(Code::Nil);
                }

                let name_str = name.as_str(db);

                // there are two types of variables: global and local, they are compiled differently
                // they are distinguished by the lexical scope depth
                if self.scope_depth == 0 {
                    chunk.emit_byte(Code::GlobalVarDeclaration {
                        name: name_str.to_string(),
                    });
                } else {
                    let local = Local::new(name_str, self.scope_depth);
                    self.locals.push(local)
                }
            }
            syntax::Stmt::Block(stmts) => {
                self.before_scope();
                for stmt in stmts {
                    self.compile_stmt(db, stmt, chunk);
                }
                self.after_scope(chunk);
            }
            syntax::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                //        ┌────────────────────┐
                //        │condition expression│
                //        └────────────────────┘
                //    ┌─── JUMP_IF_FALSE
                //    │    POP
                //    │   ┌─────────────────────┐
                //    │   │then branch statement│
                //    │   └─────────────────────┘
                // ┌──┼─── JUMP
                // │  └──► POP
                // │      ┌─────────────────────┐
                // │      │else branch statement│
                // │      └─────────────────────┘
                // └─────► continues...

                self.compile_expr(db, condition, chunk);

                // if the condition is false, jump to the end of the then branch,
                // but we don't know where the end of the then branch is yet, so we emit a placeholder
                let jump_to_the_end_of_then_branch = chunk.emit_byte(Code::JumpIfFalse(0));

                // this `pop` is only executed if the condition is true,
                // it pops the value of the condition expression
                chunk.emit_byte(Code::Pop);

                self.compile_stmt(db, then_branch, chunk);

                // after executing the then branch, we jump to the end of the else branch,
                // but we don't know where the end of the else branch is yet, so we emit a placeholder
                let jump_to_the_end_of_else_branch = chunk.emit_byte(Code::Jump(0));

                // after the then branch, we know where the end of the then branch is,
                // so we can fill in the placeholder
                self.patch_jump(jump_to_the_end_of_then_branch, chunk);

                // this `pop` is only executed if the condition is false,
                // it pops the value of the condition expression
                chunk.emit_byte(Code::Pop);

                if let Some(else_branch) = else_branch {
                    self.compile_stmt(db, else_branch, chunk);
                }

                // after compiling the else branch, we know where the end of the else branch is,
                // so we can fill in the placeholder
                self.patch_jump(jump_to_the_end_of_else_branch, chunk);
            }
            syntax::Stmt::While { condition, body } => {
                //         ┌────────────────────┐
                // ┌─────► │condition expression│
                // │       └────────────────────┘
                // │  ┌──  JUMP_IF_FALSE
                // │  │    POP
                // │  │    ┌───────────────────┐
                // │  │    │body statement list│
                // │  │    └───────────────────┘
                // └──┼─── JUMP
                //    └──► POP
                //         continues...

                // the offset of the beginning of the condition expression
                let condition_offset = chunk.len();

                self.compile_expr(db, condition, chunk);

                // if the condition is false, jump to the end of the while loop,
                // but we don't know where the end of the while loop is yet, so we emit a placeholder
                let jump_to_the_end_of_while_loop = chunk.emit_byte(Code::JumpIfFalse(0));

                // this `pop` is only executed if the condition is true,
                // it pops the value of the condition expression
                chunk.emit_byte(Code::Pop);

                self.compile_stmt(db, body, chunk);

                // after executing the body, we jump to the beginning of the condition expression,
                chunk.emit_byte(Code::Jump(condition_offset));

                // after compiling the body, we know where the end of the while loop is,
                // so we can fill in the placeholder
                self.patch_jump(jump_to_the_end_of_while_loop, chunk);

                // this `pop` is only executed if the condition is false,
                // it pops the value of the condition expression
                chunk.emit_byte(Code::Pop);
            }
            syntax::Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                //        ┌─────────────────────┐
                //        │initializer statement│
                //        └─────────────────────┘
                //        ┌─────────────────────┐
                //    ┌─► │condition expression │
                //    │   └─────────────────────┘
                // ┌──┼── JUMP_IF_FALSE
                // │  │   POP
                // │  │   ┌─────────────────────┐
                // │  │   │body statement list  │
                // │  │   └─────────────────────┘
                // │  │   ┌─────────────────────┐
                // │  │   │increment expression │
                // │  │   └─────────────────────┘
                // │  └── JUMP
                // │      POP
                // └────► continues...

                if let Some(initializer) = initializer {
                    self.compile_stmt(db, initializer, chunk);
                }

                // the offset of the beginning of the condition expression
                let condition_offset = chunk.len();

                if let Some(condition) = condition {
                    self.compile_expr(db, condition, chunk);
                } else {
                    // if there is no condition, we treat it as `true`
                    chunk.emit_byte(Code::True);
                }

                // if the condition is false, jump to the end of the for loop,
                // but we don't know where the end of the for loop is yet, so we emit a placeholder
                let jump_to_the_end_of_for_loop = chunk.emit_byte(Code::JumpIfFalse(0));

                // this `pop` is only executed if the condition is true,
                // it pops the value of the condition expression
                chunk.emit_byte(Code::Pop);

                self.compile_stmt(db, body, chunk);

                if let Some(increment) = increment {
                    self.compile_expr(db, increment, chunk);
                    chunk.emit_byte(Code::Pop);
                }

                // after executing the body, we jump to the beginning of the condition expression,
                chunk.emit_byte(Code::Jump(condition_offset));

                // after compiling the body, we know where the end of the for loop is,
                // so we can fill in the placeholder
                self.patch_jump(jump_to_the_end_of_for_loop, chunk);

                // this for loop is over, so we pop the value of the condition expression
                chunk.emit_byte(Code::Pop);
            }
        }
    }

    fn compile_expr(&mut self, db: &dyn crate::Db, expr: &syntax::Expr, chunk: &mut Chunk) {
        match expr {
            syntax::Expr::NumberLiteral(word) => {
                let word_str = word.as_str(db);
                let value = word_str.parse::<f64>().unwrap();
                chunk.emit_byte(Code::Constant(value.into()));
            }
            syntax::Expr::StringLiteral(word) => {
                let word_str = word.as_str(db);
                let value = word_str.to_string();
                chunk.emit_byte(Code::String(value));
            }
            syntax::Expr::BooleanLiteral(value) => {
                if *value {
                    chunk.emit_byte(Code::True);
                } else {
                    chunk.emit_byte(Code::False);
                }
            }
            syntax::Expr::NilLiteral => todo!(),
            syntax::Expr::BinaryOp(left, op, right) => {
                self.compile_expr(db, left, chunk);
                self.compile_expr(db, right, chunk);
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
                };
            }
            syntax::Expr::UnaryOp(op, expr) => {
                self.compile_expr(db, expr, chunk);
                match op {
                    syntax::Op::Minus => chunk.emit_byte(Code::Negate),
                    syntax::Op::Bang => chunk.emit_byte(Code::Not),
                    _ => todo!(),
                };
            }
            syntax::Expr::Parenthesized(_) => todo!(),
            syntax::Expr::Variable(word) => {
                let name = word.as_str(db);
                if let Some(index) = self.resolve_local(name) {
                    chunk.emit_byte(Code::ReadLocalVariable {
                        index_in_stack: index,
                    })
                } else {
                    chunk.emit_byte(Code::ReadGlobalVariable {
                        name: name.to_string(),
                    })
                };
            }
            syntax::Expr::Assign { name, value } => {
                self.compile_expr(db, value, chunk);
                let name_str = name.as_str(db);
                chunk.emit_byte(Code::Assign(name_str.to_string()));
            }
            syntax::Expr::LogicalAnd(left, right) => {
                //      ┌───────────────┐
                //      │left expression│
                //      └───────────────┘
                // ┌──── JUMP_IF_FALSE
                // │     POP
                // │    ┌────────────────┐
                // │    │right expression│
                // │    └────────────────┘
                // └───► continues...
                self.compile_expr(db, left, chunk);

                // if the left branch is false, jump to the end of the right branch,
                // which means we don't execute the right branch
                // for example, `false and 1 / 0` will not cause a division by zero error
                let jump_to_the_end_of_right_branch = chunk.emit_byte(Code::JumpIfFalse(0));

                // this `pop` is only executed if the left branch is true
                chunk.emit_byte(Code::Pop);

                self.compile_expr(db, right, chunk);

                // after executing the right branch, we know where the end of the right branch is,
                // so we can fill in the placeholder
                self.patch_jump(jump_to_the_end_of_right_branch, chunk);
            }
            syntax::Expr::LogicalOr(left, right) => {
                //       ┌───────────────┐
                //       │left expression│
                //       └───────────────┘
                //    ┌── JUMP_IF_FASLE
                // ┌──┼── JUMP
                // │  └─► POP
                // │     ┌────────────────┐
                // │     │right expression│
                // │     └────────────────┘
                // └────► continues...
                self.compile_expr(db, left, chunk);
                let jump_if_left_is_false = chunk.emit_byte(Code::JumpIfFalse(0));

                // if the left branch is true, we don't need to execute the right branch
                let jump_if_left_is_true = chunk.emit_byte(Code::Jump(0));
                self.patch_jump(jump_if_left_is_false, chunk);
                self.compile_expr(db, right, chunk);
                self.patch_jump(jump_if_left_is_true, chunk);
            }
        }
    }

    fn before_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn after_scope(&mut self, chunk: &mut Chunk) {
        self.scope_depth -= 1;
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.locals.pop();
            chunk.emit_byte(Code::Pop);
        }
    }

    // returns the index of the local variable
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    // patch a jump instruction with the current offset
    fn patch_jump(&mut self, jump: usize, chunk: &mut Chunk) {
        let offset = chunk.len();
        let jump = chunk.read_byte_mut(jump);
        if let Code::Jump(ip) | Code::JumpIfFalse(ip) = jump {
            *ip = offset;
        }
    }
}
