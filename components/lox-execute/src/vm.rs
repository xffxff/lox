use std::collections::HashMap;

use lox_ir::bytecode::{self, Function, Upvalue};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Closure {
        function: Function,
        upvalues: Vec<Upvalue>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::String(s) => write!(f, "{}", s),
            Value::Closure { function, .. } => write!(f, "<func {}>", function.name),
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
            Value::Closure { function, .. } => write!(f, "<func {}>", function.name),
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
struct UpvalueObject {
    value: Option<Value>,
    location_in_stack: usize,
}

#[derive(Debug, Clone)]
struct CallFrame {
    function: Function,
    ip: usize,
    fp: usize,

    // the absolute index of the upvalue in the stack
    upvalues: Vec<UpvalueObject>,
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

    heap: generational_arena::Arena<Value>,

    pub stack: Vec<generational_arena::Index>,

    // global variables
    globals: HashMap<String, Value>,

    // output buffer
    pub output: String,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            frames: vec![],
            heap: generational_arena::Arena::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            output: String::new(),
        }
    }

    pub fn push_frame(&mut self, function: Function, upvalues: Vec<Upvalue>) {
        let arity = function.arity;

        // convert the upvalues to absolute indices in the stack
        let upvalues = if let Some(current_frame) = self.current_frame() {
            upvalues
                .into_iter()
                .map(|upvalue| {
                    if upvalue.is_local {
                        UpvalueObject {
                            value: None,
                            location_in_stack: current_frame.fp + upvalue.index,
                        }
                    } else {
                        current_frame.upvalues[upvalue.index].clone()
                    }
                })
                .collect()
        } else {
            vec![]
        };

        let frame = CallFrame {
            function,
            ip: 0,
            fp: self.stack.len() - arity,
            upvalues,
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
    pub(crate) fn step<F>(&mut self, mut step_inspect: Option<F>) -> ControlFlow
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
        tracing::debug!("stack: {:?}", &self.stack[frame.fp..]);
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
                let value_idx = self.stack[frame.fp + index_in_stack];
                let value = self.heap[value_idx].clone();
                self.push(value);
            }
            bytecode::Code::WriteLocalVariable { index_in_stack } => {
                let value = self.peek();
                let value_idx = self.stack[frame.fp + index_in_stack];
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
            // bytecode::Code::Function(function) => {
            //     let function = Value::Function(Function::from(function));
            //     self.push(function);
            // }
            bytecode::Code::Call { arity } => {
                let closure = self.peek_n_from_top(arity);
                match closure {
                    Value::Closure { function, upvalues } => {
                        self.push_frame(function.clone(), upvalues.clone());
                    }
                    _ => panic!("Cannot call {:?}", closure),
                }
            }
            bytecode::Code::Closure { function, upvalues } => {
                let closure = Value::Closure {
                    function: function.clone(),
                    upvalues: upvalues.clone(),
                };
                self.push(closure);
            }
            bytecode::Code::ReadUpvalue { index } => {
                let upvalue = &frame.upvalues[index];
                if let Some(value) = &upvalue.value {
                    self.push(value.clone());
                } else {
                    let value_idx = self.stack[upvalue.location_in_stack].clone();
                    self.push(self.heap[value_idx].clone())
                }
            }
            bytecode::Code::WriteUpvalue { index } => {
                let upvalue = &frame.upvalues[index];
                let value = self.peek();
                let value_idx = self.stack[upvalue.location_in_stack];
                self.heap[value_idx] = value.clone();
            }
            bytecode::Code::CloseUpvalue => {
                // self.close_upvalue(&mut frame, self.stack.len() - 1);
                // self.pop();
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

    fn close_upvalue(&mut self, frame: &mut CallFrame, location_in_stack: usize) {
        let upvalue_index = frame
            .upvalues
            .iter()
            .position(|upvalue| upvalue.location_in_stack == location_in_stack)
            .unwrap();
        let upvalue = frame.upvalues.get_mut(upvalue_index).unwrap();
        let value_idx = self.stack[location_in_stack];
        // upvalue.location_in_stack = value_idx;
        // upvalue.value = Some(value);
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
