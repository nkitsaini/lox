use crate::{
    chunk::{Chunk, OpCode},
    prelude::ValuePrinter,
};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);
    let mut last_line = None;
    for (operation, line) in &chunk.code {
        match last_line {
            Some(x) if x == line => print!("   | "),
            _ => print!("{:>4} ", line),
        }
        last_line = Some(line);
        operation.show_disassemble(chunk);
    }
}

impl OpCode {
    pub fn show_disassemble(&self, chunk: &Chunk) {
        use OpCode::*;
        match self {
            Return => self.simple_instruction("OP_RETURN"),
            Constant { location } => self.constant_instruction(chunk, "OP_CONSTANT", *location),
            Negate => self.simple_instruction("OP_NEGATE"),

            Nil => self.simple_instruction("OP_NIL"),
            True => self.simple_instruction("OP_TRUE"),
            False => self.simple_instruction("OP_FALSE"),
            Not => self.simple_instruction("OP_NOT"),

            Equal => self.simple_instruction("OP_EQUAL"),
            Less => self.simple_instruction("OP_LESS    "),
            Greater => self.simple_instruction("OP_GREATER"),

            Add => self.simple_instruction("OP_ADD"),
            Multiply => self.simple_instruction("OP_MULTIPLY"),
            Subtract => self.simple_instruction("OP_SUBTRACT"),
            Divide => self.simple_instruction("OP_DIVIDE"),

            Print => self.simple_instruction("OP_PRINT"),
            Pop => self.simple_instruction("OP_POP"),

            DefineGlobal { location } => {
                self.constant_instruction(chunk, "OP_DEFINE_GLOBAL", *location)
            }
            GetGlobal { location } => self.constant_instruction(chunk, "OP_GET_GLOBAL", *location),
            SetGlobal { location } => self.constant_instruction(chunk, "OP_SET_GLOBAL", *location),
        }
    }

    fn constant_instruction(&self, chunk: &Chunk, name: &str, location: u8) {
        print!("{:<16} {:<4}", name, location);
        let value = chunk.constants[location as usize].clone();
        value.print();
        println!();
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
