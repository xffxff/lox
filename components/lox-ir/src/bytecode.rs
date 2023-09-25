#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Code {
    Return,
    Constant(eq_float::F64),
    True,
    False,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    String(String),
    Print,
    GlobalVarDeclaration(String), // name of the variable
    GlobalVariable(String),       // name of the variable
    LocalVariable(usize),         // index of the variable in the stack
    Nil,
    Assign(String),
    Pop,
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

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}
