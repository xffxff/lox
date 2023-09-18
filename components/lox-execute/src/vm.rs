use lox_ir::bytecode;


pub struct VM {
    chunk: bytecode::Chunk,
    ip: usize,

    // FIXME: This should be a stack of values, not a stack of f64s.
    stack: Vec<f64>, 
}

impl VM {
    pub fn interpret(&mut self) {
        loop {
            let instruction = self.read_byte();
            match instruction {
                bytecode::Code::Return => return,
                bytecode::Code::Constant(value) => {
                    self.push(value.into())
                },
                bytecode::Code::Add => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                },
                bytecode::Code::Subtract => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                },
                bytecode::Code::Multiply => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                },
                bytecode::Code::Divide => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                },
            }
        }
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

