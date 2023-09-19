use lox_ir::bytecode;

pub struct VM {
    chunk: bytecode::Chunk,
    ip: usize,

    // FIXME: This should be a stack of values, not a stack of f64s.
    stack: Vec<f64>,
}

impl VM {
    pub fn new(chunk: bytecode::Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    // FIXME: `interpret` should not return a `f64`, but for now it's convenient as
    //  our compiler is more or less a calculator.
    pub fn interpret(&mut self) -> f64 {
        loop {
            if self.chunk.len() <= self.ip {
                break;
            }
            tracing::debug!("ip: {}", self.ip);
            tracing::debug!("stack: {:?}", self.stack);
            let instruction = self.read_byte();
            match instruction {
                bytecode::Code::Return => break,
                bytecode::Code::Constant(value) => self.push(value.into()),
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
                    // FIXME: This should be a boolean, not a float.
                    self.push(1.0);
                },
                bytecode::Code::False => {
                    // FIXME: This should be a boolean, not a float.
                    self.push(0.0);
                },
            }
        }
        self.stack.pop().unwrap()
    }

    fn read_byte(&mut self) -> bytecode::Code {
        let byte = self.chunk.read_byte(self.ip);
        self.ip += 1;
        byte
    }

    fn pop(&mut self) -> f64 {
        self.stack.pop().unwrap()
    }

    fn push(&mut self, value: f64) {
        self.stack.push(value);
    }
}
