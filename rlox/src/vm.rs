// use crate::{compiler::compile, prelude::*};
use crate::{compiler::Compiler, prelude::*};
use smallvec;

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Chunk,

    // Huh, the book says looking by index is slower them
    // looking by index. Why could that be? Due to additions?
    ip: usize,

    stack: smallvec::SmallVec<[Value; STACK_MAX]>,
}

macro_rules! binary_op {
    ($vm:ident, $op:tt) => {{
        let a = $vm.stack.pop().unwrap();
        let b = $vm.stack.pop().unwrap();
        $vm.stack.push(b $op a);
    }};
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: smallvec::SmallVec::new(),
        }
    }

    pub fn interpret(source: &str) -> InterpreterResult {
        let chunk: Option<Chunk> = Compiler::compile(source);
        let chunk = match chunk {
            Some(x) => x,
            None => return Err(InterpreterError::CompileError),
        };

        let mut vm = Self::new(chunk);
        let result = vm.run();
        return result;
    }

    fn run(&mut self) -> InterpreterResult {
        loop {
            let instruction = self.chunk.code[self.ip].0;

            // Debug stuff
            #[cfg(feature = "trace_execution")]
            {
                print!("        stack: ");
                for value in self.stack.iter() {
                    print!("[ ");
                    value.print();
                    print!(" ]");
                }
                println!();
                instruction.show_disassemble(&self.chunk);
            }

            use OpCode::*;

            // Execution
            self.ip += 1;
            match instruction {
                Return => {
                    self.stack.pop().unwrap().print();
                    println!();
                    return Ok(());
                }
                Constant { location } => {
                    let constant = self.chunk.constants[location as usize];
                    self.stack.push(constant);
                }
                Negate => {
                    let val = -self.stack.pop().unwrap();
                    self.stack.push(val);
                }
                Add => binary_op!(self, +),
                Multiply => binary_op!(self, *),
                Subtract => binary_op!(self, -),
                Divide => binary_op!(self, /),
            }
        }
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    CompileError,
    RuntimeError,
}

pub type InterpreterResult = Result<(), InterpreterError>;
