
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Code {
    Return,
    Constant(eq_float::F64),
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct Chunk {
    code: Vec<Code>,
}

impl Chunk {
    pub fn emit_byte(&mut self, byte: Code) {
        self.code.push(byte);
    }

    pub fn read_byte(&self, ip: usize) -> Code {
        self.code[ip].clone()
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }
}