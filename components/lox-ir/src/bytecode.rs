
pub enum Code {
    Return,
    Constant(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct Chunk {
    code: Vec<Code>,
}