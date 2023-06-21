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
            Constant { location } => {
                print!("{:<16} {:<4}", "OP_CONSTANT", location);
                let value = chunk.constants[*location as usize];
                value.print();
                println!();
            }
            Negate => self.simple_instruction("OP_NEGATE"),
            Add => self.simple_instruction("OP_ADD"),
            Multiply => self.simple_instruction("OP_MULTIPLY"),
            Subtract => self.simple_instruction("OP_SUBTRACT"),
            Divide => self.simple_instruction("OP_DIVIDE"),
        }
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
