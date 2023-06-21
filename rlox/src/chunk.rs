use crate::prelude::Value;

#[derive(Clone, Copy)]
pub enum OpCode {
    Return,
    // TODO(memory): This makes every opcode consume 2 bytes.
    // Maybe store [u8] as chunk instead of [OpCode] like book does?
    Constant { location: u8 },
    Negate,

    Add,
    Subtract,
    Multiply,
    Divide,
}

type LineNo = usize;
pub struct Chunk {
    pub code: Vec<(OpCode, LineNo)>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        return Self {
            code: vec![],
            constants: vec![],
        };
    }

    pub fn write(&mut self, code: OpCode, line: usize) {
        self.code.push((code, line));
    }

    pub fn write_constant(&mut self, value: Value, line: usize) -> u8 {
        self.constants.push(value);
        self.write(
            OpCode::Constant {
                // TODO: handle overflow
                location: (self.constants.len() - 1) as u8,
            },
            line,
        );
        return (self.constants.len() - 1) as u8;
    }
}
