use std::collections::HashMap;

use lox_ir::bytecode;

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    arity: usize,
    chunk: bytecode::Chunk,
}

impl From<bytecode::Function> for Function {
    fn from(f: bytecode::Function) -> Self {
        Self {
            name: f.name,
            arity: f.arity,
            chunk: f.chunk,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Function(Function),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(Function) => todo!(),
        }
    }
}

impl From<f64> for Value {
    fn from(num: f64) -> Self {
        Value::Number(num)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(a.clone() + b),
            _ => panic!("Cannot add {:?} and {:?}", self, rhs),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Cannot subtract {:?} and {:?}", self, rhs),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Cannot multiply {:?} and {:?}", self, rhs),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Cannot divide {:?} and {:?}", self, rhs),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(a) => Value::Number(-a),
            _ => panic!("Cannot negate {:?}", self),
        }
    }
}

impl std::ops::Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(a) => Value::Boolean(!a),
            _ => panic!("Cannot negate {:?}", self),
        }
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ControlFlow {
    Next,
    Done,
}

#[derive(Debug, Clone)]
struct CallFrame {
    function: Function,
    ip: usize,
    fp: usize,
}

impl CallFrame {
    fn read_byte(&mut self) -> bytecode::Code {
        let byte = self.function.chunk.read_byte(self.ip);
        self.ip += 1;
        byte
    }
}

pub struct VM {
    frames: Vec<CallFrame>,

    pub stack: Vec<Value>,

    // global variables
    globals: HashMap<String, Value>,

    // output buffer
    pub output: String,
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            stack: Vec::new(),
            globals: HashMap::new(),
            output: String::new(),
        }
    }

    pub fn push_frame(&mut self, function: Function) {
        let arity = function.arity;
        let frame = CallFrame {
            function: function,
            ip: 0,
            fp: self.stack.len() - arity,
        };
        tracing::debug!("pushing frame: {:?}", frame);
        self.frames.push(frame);
    }

    fn current_frame(&self) -> CallFrame {
        self.frames.last().unwrap().clone()
    }

    fn update_frame(&mut self, frame_index: usize, frame: CallFrame) {
        self.frames[frame_index] = frame;
    }

    // `step_inspect` is a callback that is called after each instruction is executed.
    //  It is useful for debugging.
    pub(crate) fn step<F>(&mut self, mut step_inspect: Option<F>) -> ControlFlow
    where
        F: FnMut(bytecode::Code, &VM),
    {
        let mut frame = self.current_frame();
        let frame_index = self.frames.len() - 1;
        tracing::debug!("current frame: {:#?}", frame);
        if frame.function.chunk.len() <= frame.ip {
            return ControlFlow::Done;
        }
        let instruction = frame.read_byte();
        tracing::debug!("ip: {}", frame.ip);
        tracing::debug!("stack: {:?}", self.stack);
        tracing::debug!("instruction: {:?}", instruction);
        match instruction.clone() {
            bytecode::Code::Return => return ControlFlow::Done,
            bytecode::Code::Constant(value) => self.push(value.0),
            bytecode::Code::Add => {
                let b = self.pop();
                let a = self.pop();
                self.push(a + b);
            }
            bytecode::Code::Subtract => {
                let b = self.pop();
                let a = self.pop();
                self.push(a - b);
            }
            bytecode::Code::Multiply => {
                let b = self.pop();
                let a = self.pop();
                self.push(a * b);
            }
            bytecode::Code::Divide => {
                let b = self.pop();
                let a = self.pop();
                self.push(a / b);
            }
            bytecode::Code::True => {
                self.push(true);
            }
            bytecode::Code::False => {
                self.push(false);
            }
            bytecode::Code::Negate => {
                let a = self.pop();
                self.push(-a);
            }
            bytecode::Code::Not => {
                let a = self.pop();
                self.push(!a);
            }
            bytecode::Code::Equal => {
                let b = self.pop();
                let a = self.pop();
                self.push(a == b);
            }
            bytecode::Code::NotEqual => {
                let b = self.pop();
                let a = self.pop();
                self.push(a != b);
            }
            bytecode::Code::Greater => {
                let b = self.pop();
                let a = self.pop();
                self.push(a > b);
            }
            bytecode::Code::GreaterEqual => {
                let b = self.pop();
                let a = self.pop();
                self.push(a >= b);
            }
            bytecode::Code::Less => {
                let b = self.pop();
                let a = self.pop();
                self.push(a < b);
            }
            bytecode::Code::LessEqual => {
                let b = self.pop();
                let a = self.pop();
                self.push(a <= b);
            }
            bytecode::Code::String(s) => {
                self.push(s);
            }
            bytecode::Code::Print => {
                let value = self.pop();
                self.print(&format!("{}", value));
            }
            bytecode::Code::GlobalVarDeclaration { name } => {
                let value = self.pop();
                self.globals.insert(name, value);
            }
            bytecode::Code::Nil => {
                self.push(Value::Nil);
            }
            bytecode::Code::ReadGlobalVariable { name } => {
                let value = self.globals.get(&name).expect("variable not found");
                self.push(value.clone());
            }
            bytecode::Code::Assign(name) => {
                let value = self.peek();
                self.globals.insert(name, value.clone());
            }
            bytecode::Code::ReadLocalVariable { index_in_stack } => {
                let value = self.stack[frame.fp + index_in_stack].clone();
                self.push(value);
            }
            bytecode::Code::Pop => {
                self.pop();
            }
            bytecode::Code::JumpIfFalse(ip) => {
                let value = self.peek();
                if value == &Value::Boolean(false) {
                    frame.ip = ip;
                }
            }
            bytecode::Code::Jump(ip) => {
                frame.ip = ip;
            }
            bytecode::Code::Function(function) => {
                let function = Value::Function(Function::from(function));
                self.push(function);
            }
            bytecode::Code::Call { arity } => {
                let function = self.peek_n_from_top(arity);
                match function {
                    Value::Function(function) => {
                        self.push_frame(function.clone());
                    }
                    _ => panic!("Cannot call {:?}", function),
                }
            }
        }
        if let Some(step_inspect) = &mut step_inspect {
            step_inspect(instruction, self);
        }
        self.update_frame(frame_index, frame);
        ControlFlow::Next
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self) -> &Value {
        self.stack.last().unwrap()
    }

    fn peek_n_from_top(&self, n: usize) -> &Value {
        &self.stack[self.stack.len() - n - 1]
    }

    fn push<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        self.stack.push(value.into());
    }

    fn print(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }
}
