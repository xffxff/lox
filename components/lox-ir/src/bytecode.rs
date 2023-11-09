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
    GlobalVarDeclaration {
        name: String, // name of the variable
    },
    ReadGlobalVariable {
        name: String, // name of the variable
    },
    ReadLocalVariable {
        index_in_stack: usize, // index of the variable in the stack
    },
    Nil,
    WriteGlobalVariable {
        name: String, // name of the variable
    },
    WriteLocalVariable {
        index_in_stack: usize, // index of the variable in the stack
    },
    Pop,
    JumpIfFalse(usize),
    Jump(usize),
    Function(crate::function::Function),
    Call {
        arity: usize,
    },
    CloseUpvalue,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct Chunk {
    code: Vec<Code>,
}

impl Chunk {
    pub fn emit_byte(&mut self, byte: Code) -> usize {
        tracing::debug!(?byte, "emitting byte");
        self.code.push(byte);
        self.len() - 1
    }

    pub fn read_byte(&self, ip: usize) -> Code {
        self.code[ip].clone()
    }

    pub fn read_byte_mut(&mut self, ip: usize) -> &mut Code {
        &mut self.code[ip]
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct CompiledFunction {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}
