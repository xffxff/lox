use salsa::DebugWithDb;

#[derive(Clone, Debug, PartialEq, Eq)]
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
}

impl<'db> DebugWithDb<dyn crate::Db + 'db> for Code {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        match self {
            Code::Return => write!(f, "return"),
            Code::Constant(c) => write!(f, "constant({})", c),
            Code::True => write!(f, "true"),
            Code::False => write!(f, "false"),
            Code::Add => write!(f, "add"),
            Code::Subtract => write!(f, "subtract"),
            Code::Multiply => write!(f, "multiply"),
            Code::Divide => write!(f, "divide"),
            Code::Negate => write!(f, "negate"),
            Code::Not => write!(f, "not"),
            Code::Equal => write!(f, "equal"),
            Code::NotEqual => write!(f, "not_equal"),
            Code::Greater => write!(f, "greater"),
            Code::GreaterEqual => write!(f, "greater_equal"),
            Code::Less => write!(f, "less"),
            Code::LessEqual => write!(f, "less_equal"),
            Code::String(s) => write!(f, "string({:?})", s),
            Code::Print => write!(f, "print"),
            Code::GlobalVarDeclaration { name } => write!(f, "global_var_declaration({})", name),
            Code::ReadGlobalVariable { name } => write!(f, "read_global_variable({})", name),
            Code::ReadLocalVariable { index_in_stack } => {
                write!(f, "read_local_variable({})", index_in_stack)
            }
            Code::Nil => write!(f, "nil"),
            Code::WriteGlobalVariable { name } => write!(f, "write_global_variable({})", name),
            Code::WriteLocalVariable { index_in_stack } => {
                write!(f, "write_local_variable({})", index_in_stack)
            }
            Code::Pop => write!(f, "pop"),
            Code::JumpIfFalse(ip) => write!(f, "jump_if_false({})", ip),
            Code::Jump(ip) => write!(f, "jump({})", ip),
            Code::Function(function) => write!(f, "function({:?})", &function.debug(db)),
            Code::Call { arity } => write!(f, "call({})", arity),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct Chunk {
    code: Vec<Code>,
}

impl<'db> DebugWithDb<dyn crate::Db + 'db> for Chunk {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        f.debug_list()
            .entries(self.code.iter().map(|c| c.debug(db)))
            .finish()
    }
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

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct CompiledFunction {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
}

impl DebugWithDb<dyn crate::Db> for CompiledFunction {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &dyn crate::Db,
        _include_all_fields: bool,
    ) -> std::fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("chunk", &self.chunk.debug(db))
            .finish()
    }
}
