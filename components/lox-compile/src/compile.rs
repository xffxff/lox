use std::{cell::RefCell, rc::Rc};

use lox_ir::{
    bytecode::{Chunk, Code, Function},
    input_file::InputFile,
    syntax,
};

#[salsa::tracked]
pub fn compile_file(db: &dyn crate::Db, input_file: InputFile) -> Function {
    let stmts = lox_parse::parse_file(db, input_file);
    let mut chunk = Chunk::default();
    let compiler = Compiler::default();
    let compiler = Rc::new(RefCell::new(compiler));
    for stmt in stmts {
        compile_stmt(compiler.clone().clone(), db, stmt, &mut chunk);
    }
    Function {
        name: "main".to_string(),
        arity: 0,
        chunk,
    }
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

    // An enclosing compiler is one level above the current compiler.
    // Each function has its own compiler, and we need to pass the enclosing compiler
    // to the function's compiler to enable access to the variables in the enclosing scope.
    // Specifically, the enclosing scope is necessary to support closures
    enclosing: Option<Rc<RefCell<Compiler>>>,
}

impl Compiler {
    fn new(enclosing: Rc<RefCell<Compiler>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            ..Default::default()
        }
    }
}

fn compile_stmt(
    compiler: Rc<RefCell<Compiler>>,
    db: &dyn crate::Db,
    stmt: &syntax::Stmt,
    chunk: &mut Chunk,
) {
    match stmt {
        syntax::Stmt::Expr(expr) => {
            compile_expr(compiler.clone(), db, expr, chunk);
            chunk.emit_byte(Code::Pop);
        }
        syntax::Stmt::Print(expr) => {
            compile_expr(compiler.clone(), db, expr, chunk);
            chunk.emit_byte(Code::Print);
        }
        syntax::Stmt::VariableDeclaration { name, initializer } => {
            if let Some(initializer) = initializer {
                compile_expr(compiler.clone(), db, initializer, chunk);
            } else {
                chunk.emit_byte(Code::Nil);
            }

            let name_str = name.as_str(db);

            // there are two types of variables: global and local, they are compiled differently
            // they are distinguished by the lexical scope depth
            if compiler.borrow().scope_depth == 0 {
                chunk.emit_byte(Code::GlobalVarDeclaration {
                    name: name_str.to_string(),
                });
            } else {
                let local = Local::new(name_str, compiler.borrow().scope_depth);
                compiler.borrow_mut().locals.push(local)
            }
        }
        syntax::Stmt::Block(stmts) => {
            before_scope(compiler.clone());
            for stmt in stmts {
                compile_stmt(compiler.clone(), db, stmt, chunk);
            }
            after_scope(compiler.clone(), chunk);
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

            compile_expr(compiler.clone(), db, condition, chunk);

            // if the condition is false, jump to the end of the then branch,
            // but we don't know where the end of the then branch is yet, so we emit a placeholder
            let jump_to_the_end_of_then_branch = chunk.emit_byte(Code::JumpIfFalse(0));

            // this `pop` is only executed if the condition is true,
            // it pops the value of the condition expression
            chunk.emit_byte(Code::Pop);

            compile_stmt(compiler.clone(), db, then_branch, chunk);

            // after executing the then branch, we jump to the end of the else branch,
            // but we don't know where the end of the else branch is yet, so we emit a placeholder
            let jump_to_the_end_of_else_branch = chunk.emit_byte(Code::Jump(0));

            // after the then branch, we know where the end of the then branch is,
            // so we can fill in the placeholder
            patch_jump(jump_to_the_end_of_then_branch, chunk);

            // this `pop` is only executed if the condition is false,
            // it pops the value of the condition expression
            chunk.emit_byte(Code::Pop);

            if let Some(else_branch) = else_branch {
                compile_stmt(compiler.clone(), db, else_branch, chunk);
            }

            // after compiling the else branch, we know where the end of the else branch is,
            // so we can fill in the placeholder
            patch_jump(jump_to_the_end_of_else_branch, chunk);
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

            compile_expr(compiler.clone(), db, condition, chunk);

            // if the condition is false, jump to the end of the while loop,
            // but we don't know where the end of the while loop is yet, so we emit a placeholder
            let jump_to_the_end_of_while_loop = chunk.emit_byte(Code::JumpIfFalse(0));

            // this `pop` is only executed if the condition is true,
            // it pops the value of the condition expression
            chunk.emit_byte(Code::Pop);

            compile_stmt(compiler.clone(), db, body, chunk);

            // after executing the body, we jump to the beginning of the condition expression,
            chunk.emit_byte(Code::Jump(condition_offset));

            // after compiling the body, we know where the end of the while loop is,
            // so we can fill in the placeholder
            patch_jump(jump_to_the_end_of_while_loop, chunk);

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
                compile_stmt(compiler.clone(), db, initializer, chunk);
            }

            // the offset of the beginning of the condition expression
            let condition_offset = chunk.len();

            if let Some(condition) = condition {
                compile_expr(compiler.clone(), db, condition, chunk);
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

            compile_stmt(compiler.clone(), db, body, chunk);

            if let Some(increment) = increment {
                compile_expr(compiler.clone(), db, increment, chunk);
                chunk.emit_byte(Code::Pop);
            }

            // after executing the body, we jump to the beginning of the condition expression,
            chunk.emit_byte(Code::Jump(condition_offset));

            // after compiling the body, we know where the end of the for loop is,
            // so we can fill in the placeholder
            patch_jump(jump_to_the_end_of_for_loop, chunk);

            // this for loop is over, so we pop the value of the condition expression
            chunk.emit_byte(Code::Pop);
        }
        syntax::Stmt::FunctionDeclaration {
            name,
            parameters,
            body,
        } => {
            let sub_compiler = Compiler::new(compiler.clone());
            let sub_compiler = Rc::new(RefCell::new(sub_compiler));
            let mut sub_chunk = Chunk::default();
            for param in parameters {
                let name_str = param.as_str(db);
                let local = Local::new(name_str, compiler.borrow().scope_depth);
                sub_compiler.borrow_mut().locals.push(local);
            }
            compile_stmt(sub_compiler, db, body, &mut sub_chunk);
            let name_str = name.as_str(db);
            let function = Function {
                name: name_str.to_string(),
                arity: parameters.len(),
                chunk: sub_chunk,
            };
            let closure = Code::Closure {
                function,
                upvalues: vec![],
            };
            chunk.emit_byte(closure);
            // there are two types of variables: global and local, they are compiled differently
            // they are distinguished by the lexical scope depth
            if compiler.borrow().scope_depth == 0 {
                chunk.emit_byte(Code::GlobalVarDeclaration {
                    name: name_str.to_string(),
                });
            } else {
                let local = Local::new(name_str, compiler.borrow().scope_depth);
                compiler.borrow_mut().locals.push(local)
            }
        }
        syntax::Stmt::Return(expr) => {
            if let Some(expr) = expr {
                compile_expr(compiler.clone(), db, expr, chunk);
            } else {
                chunk.emit_byte(Code::Nil);
            }
            chunk.emit_byte(Code::Return);
        }
    }
}

fn compile_expr(
    compiler: Rc<RefCell<Compiler>>,
    db: &dyn crate::Db,
    expr: &syntax::Expr,
    chunk: &mut Chunk,
) {
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
            compile_expr(compiler.clone(), db, left, chunk);
            compile_expr(compiler.clone(), db, right, chunk);
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
            compile_expr(compiler.clone(), db, expr, chunk);
            match op {
                syntax::Op::Minus => chunk.emit_byte(Code::Negate),
                syntax::Op::Bang => chunk.emit_byte(Code::Not),
                _ => todo!(),
            };
        }
        syntax::Expr::Parenthesized(_) => todo!(),
        syntax::Expr::Variable(word) => {
            let name = word.as_str(db);
            if let Some(index) = resolve_local(compiler, name) {
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
            compile_expr(compiler.clone(), db, value, chunk);
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
            compile_expr(compiler.clone(), db, left, chunk);

            // if the left branch is false, jump to the end of the right branch,
            // which means we don't execute the right branch
            // for example, `false and 1 / 0` will not cause a division by zero error
            let jump_to_the_end_of_right_branch = chunk.emit_byte(Code::JumpIfFalse(0));

            // this `pop` is only executed if the left branch is true
            chunk.emit_byte(Code::Pop);

            compile_expr(compiler.clone(), db, right, chunk);

            // after executing the right branch, we know where the end of the right branch is,
            // so we can fill in the placeholder
            patch_jump(jump_to_the_end_of_right_branch, chunk);
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
            compile_expr(compiler.clone(), db, left, chunk);
            let jump_if_left_is_false = chunk.emit_byte(Code::JumpIfFalse(0));

            // if the left branch is true, we don't need to execute the right branch
            let jump_if_left_is_true = chunk.emit_byte(Code::Jump(0));
            patch_jump(jump_if_left_is_false, chunk);
            compile_expr(compiler.clone(), db, right, chunk);
            patch_jump(jump_if_left_is_true, chunk);
        }
        syntax::Expr::Call { callee, arguments } => {
            compile_expr(compiler.clone(), db, callee, chunk);
            for arg in arguments {
                compile_expr(compiler.clone(), db, arg, chunk);
            }
            chunk.emit_byte(Code::Call {
                arity: arguments.len(),
            });
        }
    }
}

fn before_scope(compiler: Rc<RefCell<Compiler>>) {
    let mut compiler = compiler.borrow_mut();
    compiler.scope_depth += 1;
}

fn after_scope(compiler: Rc<RefCell<Compiler>>, chunk: &mut Chunk) {
    let mut compiler = compiler.borrow_mut();
    compiler.scope_depth -= 1;
    while !compiler.locals.is_empty()
        && compiler.locals.last().unwrap().depth > compiler.scope_depth
    {
        compiler.locals.pop();
        chunk.emit_byte(Code::Pop);
    }
}

// returns the index of the local variable
fn resolve_local(compiler: Rc<RefCell<Compiler>>, name: &str) -> Option<usize> {
    for (i, local) in compiler.borrow().locals.iter().enumerate().rev() {
        if local.name == name {
            return Some(i);
        }
    }
    None
}

// patch a jump instruction with the current offset
fn patch_jump(jump: usize, chunk: &mut Chunk) {
    let offset = chunk.len();
    let jump = chunk.read_byte_mut(jump);
    if let Code::Jump(ip) | Code::JumpIfFalse(ip) = jump {
        *ip = offset;
    }
}
