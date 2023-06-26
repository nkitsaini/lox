use crate::prelude::Value;

#[derive(Clone, Copy)]
pub enum OpCode {
    Return,
    // TODO(memory): This makes every opcode consume 2 bytes.
    // Maybe store [u8] as chunk instead of [OpCode] like book does?
    Constant { location: u8 },
    Negate,

    Nil,
    True,
    False,
    Not,

    Equal,
    Greater,
    Less,

    Add,
    Subtract,
    Multiply,
    Divide,

    Print,
    Pop,
    DefineGlobal { location: u8 },
    GetGlobal { location: u8 },
    SetGlobal { location: u8 },
    GetLocal { stack_idx: u8 },
    SetLocal { stack_idx: u8 },
}

type LineNo = usize;
pub struct Chunk<'a> {
    pub code: Vec<(OpCode, LineNo)>,
    pub constants: Vec<Value<'a>>,
}

impl<'a> Chunk<'a> {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
        };
    }

    pub fn write(&mut self, code: OpCode, line: usize) {
        self.code.push((code, line));
    }

    /// Add constant without any opcode
    pub fn add_constant(&mut self, value: Value<'a>) -> u8 {
        self.constants.push(value);
        return (self.constants.len() - 1) as u8;
    }
}
