use crate::{rle::RunLengthEncoded, value::Value};

#[derive(Clone, Copy)]
pub enum Op {
    Const(usize),
    DefineGlobal(usize),
    GetGlobal(usize),
    SetGlobal(usize),
    GetLocal(usize),
    SetLocal(usize),
    JumpIfFalse(usize),
    Nil,
    True,
    False,
    Not,
    Negate,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Print,
    Pop,
    Return,
}

pub struct Chunk {
    pub code: Vec<Op>,
    pub constants: Vec<Value>,
    pub lines: RunLengthEncoded<usize>,
}

impl Chunk {
    pub fn init() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: RunLengthEncoded::new(),
        }
    }
    pub fn add_op(&mut self, code: Op, line: usize) {
        self.lines.push(line);
        self.code.push(code);
    }
    pub fn add_const(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
