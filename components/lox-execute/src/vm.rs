use std::collections::HashMap;

use lox_compile::prelude::FunctionCompileExt;
use lox_ir::{
    bytecode::{self, CompiledFunction},
    function::Function,
};

#[derive(Clone)]
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
            Value::Function(func) => write!(f, "<func {:?}>", func),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(func) => write!(f, "<func {:?}>", func),
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
    function: CompiledFunction,
    ip: usize,
    fp: usize,
}

impl CallFrame {
    fn read_byte(&mut self) -> bytecode::Code {
        let byte = self.function.chunk.read_byte(self.ip);
        self.ip += 1;
        byte
    }

    fn local_variable(
        &self,
        stack: &[generational_arena::Index],
        index: usize,
    ) -> generational_arena::Index {
        stack[self.fp + index + 1]
    }
}

pub struct VM {
    frames: Vec<CallFrame>,

    heap: generational_arena::Arena<Value>,

    pub stack: Vec<generational_arena::Index>,

    // global variables
    globals: HashMap<String, Value>,

    // output buffer
    pub output: String,
}

impl VM {
    pub fn new(main: Function, db: &dyn crate::Db) -> Self {
        let function = main.compile(db);
        let frame = CallFrame {
            function,
            ip: 0,
            fp: 0,
        };

        let mut heap = generational_arena::Arena::new();
        let index_of_main = heap.insert(Value::Function(main));
        // push the value of the main function to the stack to a call to the main function,
        // making it is consistent with other function calls.
        let stack = vec![index_of_main];

        Self {
            frames: vec![frame],
            heap,
            stack,
            globals: HashMap::new(),
            output: String::new(),
        }
    }

    pub fn push_frame(&mut self, function: CompiledFunction) {
        let arity = function.arity;

        let frame = CallFrame {
            function,
            ip: 0,

            // fp points to the first value introduced by the current frame in the stack
            //
            // For example, the stack may look like this:
            //
            //            fp        stack top
            //             v            v
            //   0    1    2    3   4   5   6   7
            // |main| 4 | sum | 5 | 6 | 7 |   |   |
            //          |<-  call frame ->|
            //                | arity = 3 |
            //
            // the the fp is `6 - 3 - 1 = 2`, -1 for the function itself.
            fp: self.stack.len() - arity - 1,
        };
        tracing::debug!("pushing frame: {:?}", frame);
        self.frames.push(frame);
    }

    fn current_frame(&self) -> Option<CallFrame> {
        self.frames.last().cloned()
    }

    fn update_frame(&mut self, frame_index: usize, frame: CallFrame) {
        self.frames[frame_index] = frame;
    }

    // clear the values introduced by the current frame from the stack
    fn clear_stack(&mut self, frame: CallFrame) {
        // FIXME: As the value in the stack is just an index into the heap,
        // truncating the stack does not free the memory in the heap.
        // This is a memory leak.
        self.stack.truncate(frame.fp);
    }

    fn done_or_next(&mut self) -> ControlFlow {
        if self.frames.is_empty() {
            ControlFlow::Done
        } else {
            ControlFlow::Next
        }
    }

    // `step_inspect` is a callback that is called after each instruction is executed.
    //  It is useful for debugging.
    pub(crate) fn step<F>(&mut self, db: &dyn crate::Db, mut step_inspect: Option<F>) -> ControlFlow
    where
        F: FnMut(Option<bytecode::Code>, &VM),
    {
        let mut inspect_step = |instruction, self_| {
            if let Some(step_inspect) = &mut step_inspect {
                step_inspect(instruction, self_);
            }
        };

        let mut frame = self.current_frame().unwrap();
        let frame_index = self.frames.len() - 1;
        tracing::debug!("current frame: {:#?}", frame);
        if frame.function.chunk.len() <= frame.ip {
            // clear the values introduced by the current frame from the stack
            self.clear_stack(frame);

            // pop the current frame
            self.frames.pop();

            inspect_step(None, self);

            return self.done_or_next();
        }
        let instruction = frame.read_byte();
        tracing::debug!("ip: {}", frame.ip);
        tracing::debug!("stack: {:?}", &self.stack_values()[frame.fp..]);
        tracing::debug!("instruction: {:?}", instruction);
        match instruction.clone() {
            bytecode::Code::Return => {
                // return the value at the top of the stack
                let value = self.pop();

                // clear the values introduced by the current frame from the stack
                self.clear_stack(frame);
                // pop the current frame
                self.frames.pop();

                // push the return value to the stack
                self.push(value);

                inspect_step(Some(instruction), self);
                return self.done_or_next();
            }
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
            bytecode::Code::WriteGlobalVariable { name } => {
                let value = self.peek();
                self.globals.insert(name, value.clone());
            }
            bytecode::Code::ReadLocalVariable { index_in_stack } => {
                let value_idx = frame.local_variable(&self.stack, index_in_stack);
                let value = self.heap[value_idx].clone();
                self.push(value);
            }
            bytecode::Code::WriteLocalVariable { index_in_stack } => {
                let value = self.peek();
                let value_idx = frame.local_variable(&self.stack, index_in_stack);
                self.heap[value_idx] = value.clone();
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
            bytecode::Code::Call { arity } => {
                let closure = self.peek_n_from_top(arity);
                match closure {
                    Value::Function(function) => {
                        let compiled_function = function.compile(db);
                        self.push_frame(compiled_function);
                    }
                    _ => panic!("Cannot call {:?}", closure),
                }
            }
            bytecode::Code::Function(function) => {
                self.push(Value::Function(function));
            }
            bytecode::Code::CloseUpvalue => {
                // FIXME: As we don't remove the value in the heap created by a function frame,
                // we don't need to do anything here for closing upvalues, but this is a memory leak.
                self.pop();
            }
        }

        inspect_step(Some(instruction), self);
        self.update_frame(frame_index, frame);
        ControlFlow::Next
    }

    fn pop(&mut self) -> Value {
        let index = self.stack.pop().unwrap();
        self.heap.remove(index).unwrap()
    }

    fn peek(&self) -> &Value {
        let index = self.stack.last().unwrap();
        self.heap.get(*index).unwrap()
    }

    fn peek_n_from_top(&self, n: usize) -> &Value {
        let index = self.stack[self.stack.len() - n - 1];
        self.heap.get(index).unwrap()
    }

    fn push<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        let index = self.heap.insert(value.into());
        self.stack.push(index);
    }

    fn print(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    // Returns the values in the stack in the order they were pushed.
    // This is useful for debugging.
    pub fn stack_values(&self) -> Vec<Value> {
        self.stack
            .iter()
            .map(|index| self.heap[*index].clone())
            .collect()
    }
}
