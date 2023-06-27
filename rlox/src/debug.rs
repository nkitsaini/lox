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

            GetLocal { stack_idx } => self.byte_instruction("OP_GET_LOCAL", *stack_idx),
            SetLocal { stack_idx } => self.byte_instruction("OP_SET_LOCAL", *stack_idx),

            JumpIfFalse { target } => self.jump_instruction("OP_JUMP_IF_FALSE", *target, true),
            Jump { target } => self.jump_instruction("OP_JUMP", *target, true),
        }
    }

    fn jump_instruction(&self, name: &str, count: u16, is_forward: bool) {
        println!(
            "{:<16} -> {}",
            name,
            count as i64 * if is_forward { 1 } else { -1 }
        )
    }

    fn byte_instruction(&self, name: &str, idx: u8) {
        // TODO: currently the idx is confused with the actual location of code block in chunk
        // Fix it.
        println!("{:<16} {:<4}", name, idx);
    }

    fn constant_instruction(&self, chunk: &Chunk, name: &str, location: u8) {
        print!("{:<16} {:<4}", name, location);
        let value = chunk.constants[location as usize].clone();
        value.print(&mut std::io::stdout());
        println!();
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
