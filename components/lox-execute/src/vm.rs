use lox_ir::bytecode;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
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

pub struct VM {
    chunk: bytecode::Chunk,
    ip: usize,

    pub stack: Vec<Value>,
}

impl VM {
    pub fn new(chunk: bytecode::Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    // `step_inspect` is a callback that is called after each instruction is executed.
    //  It is useful for debugging.
    pub fn interpret<F>(&mut self, mut step_inspect: Option<F>)
    where
        F: FnMut(bytecode::Code, &VM),
    {
        loop {
            if self.chunk.len() <= self.ip {
                break;
            }
            tracing::debug!("ip: {}", self.ip);
            tracing::debug!("stack: {:?}", self.stack);
            let instruction = self.read_byte();
            match instruction.clone() {
                bytecode::Code::Return => break,
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
            }
            if let Some(step_inspect) = &mut step_inspect {
                step_inspect(instruction, self);
            }
        }
    }

    fn read_byte(&mut self) -> bytecode::Code {
        let byte = self.chunk.read_byte(self.ip);
        self.ip += 1;
        byte
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn push<T>(&mut self, value: T)
    where
        T: Into<Value>,
    {
        self.stack.push(value.into());
    }
}
