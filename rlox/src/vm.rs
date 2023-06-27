use std::io::{self, Write};

// use crate::{compiler::compile, prelude::*};
use crate::{compiler::Compiler, hashtable::HashTable, prelude::*, value::LoxObject};
use smallvec;

const STACK_MAX: usize = 256;

pub struct VM<'a, 'b, WS: Write, WE: Write> {
    chunk: Chunk<'a>,

    // Huh, the book says looking by index is slower them
    // looking by index. Why could that be? Due to additions?
    ip: usize,

    stack: smallvec::SmallVec<[Value<'a>; STACK_MAX]>,

    strings: HashTable<'a>,

    globals: HashTable<'a>,

    stdout: &'b mut WS,
    stderr: &'b mut WE,
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(x) => !x,
        Value::Number(x) => x == 0f64,
        _ => false,
    }
}

// fn values_equal(value1: Value, value2: Value) -> bool {
//     if value1.get_type() != value2.get_type() {
//         return false;
//     }

// }
macro_rules! binary_op {
    ($vm:ident, $result:path, $op:tt) => {{
        let (a, b) = match ($vm.peek(0), $vm.peek(1)) {
            (Value::Number(a), Value::Number(b)) => (a, b),
            _ => {
                $vm.runtime_error("Operands must be numbers.");
                return Err(InterpreterError::RuntimeError);
            }
        };
        $vm.stack.pop();
        $vm.stack.pop();

        // $vm.stack.push(Value::Number(b $op a));

        $vm.stack.push($result(b $op a));
    }};
}

impl<'a, 'b, WS: Write, WE: Write> VM<'a, 'b, WS, WE> {
    pub fn new(chunk: Chunk<'a>, stdout: &'b mut WS, stderr: &'b mut WE) -> Self {
        Self::new_with_strings(chunk, HashTable::new(), stdout, stderr)
    }

    pub fn empty_new(stdout: &'b mut WS, stderr: &'b mut WE) -> Self {
        Self::new(Chunk::new(), stdout, stderr)
    }

    pub fn new_with_strings(
        chunk: Chunk<'a>,
        strings: HashTable<'a>,
        stdout: &'b mut WS,
        stderr: &'b mut WE,
    ) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: smallvec::SmallVec::new(),
            strings,
            globals: HashTable::new(),
            stdout,
            stderr,
        }
    }
    pub fn interpret(&mut self, source: &'a str) -> InterpreterResult {
        let old_strings = std::mem::replace(&mut self.strings, HashTable::new());
        let (chunk, new_strings) = match Compiler::compile(source, old_strings, &mut self.stderr) {
            Some(x) => x,
            None => return Err(InterpreterError::CompileError),
        };
        self.ip = 0;
        self.chunk = chunk;
        self.strings = new_strings;
        self.run()
    }

    fn peek(&self, distance: usize) -> Value<'a> {
        return self
            .stack
            .get(self.stack.len() - 1 - distance)
            .cloned()
            .unwrap();
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
                    value.print(&mut io::stdout());
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
                    // Exit interpreter;
                    return Ok(());
                }
                Constant { location } => {
                    let constant = self.chunk.constants[location as usize].clone();
                    self.stack.push(constant);
                }
                Negate => {
                    let val = match self.peek(0) {
                        Value::Number(x) => x,
                        _ => {
                            self.runtime_error("Operand must be a number.");
                            return Err(InterpreterError::RuntimeError);
                        }
                    };
                    self.stack.push(Value::Number(val));
                }

                Nil => self.stack.push(Value::Nil),
                True => self.stack.push(Value::Bool(true)),
                False => self.stack.push(Value::Bool(false)),

                Equal => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                Greater => binary_op!(self, Value::Bool, >),
                Less => binary_op!(self, Value::Bool, <),

                Not => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(is_falsey(val)));
                }
                Add => match (self.peek(0), self.peek(1)) {
                    (Value::Number(x), Value::Number(y)) => {
                        self.stack.pop();
                        self.stack.pop();
                        self.stack.push(Value::Number(y + x));
                    }
                    (Value::Object(a), Value::Object(b)) => match (a.as_ref(), b.as_ref()) {
                        (
                            LoxObject::String { value: _, hash: _ },
                            LoxObject::String { value: _, hash: _ },
                        ) => {
                            self.concatenate();
                        }
                        _ => {
                            self.runtime_error("Operands must be two numbers or two strings.");
                            return Err(InterpreterError::RuntimeError);
                        }
                    },
                    _ => {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(InterpreterError::RuntimeError);
                    }
                },
                // Add => binary_op!(self, Value::Number, +),
                Multiply => binary_op!(self, Value::Number, *),
                Subtract => binary_op!(self, Value::Number, -),
                Divide => binary_op!(self, Value::Number, /),

                Print => {
                    self.stack.pop().unwrap().print(self.stdout);
                    writeln!(self.stdout);
                }

                Pop => {
                    self.stack.pop().unwrap();
                }

                DefineGlobal { location } => {
                    let name = self.read_constant(location).as_object().unwrap().clone();
                    let val = self.peek(0);
                    self.globals.set(name, val);
                    self.stack.pop().unwrap();
                }
                GetGlobal { location } => {
                    let name = self.read_constant(location).as_object().unwrap().clone();
                    match self.globals.get(&name) {
                        None => {
                            self.runtime_error(&format!(
                                "Undefined variable '{}'",
                                name.as_string().unwrap().0
                            ));
                            return Err(InterpreterError::RuntimeError);
                        }
                        Some(x) => self.stack.push(x.clone()),
                    }
                }
                SetGlobal { location } => {
                    let name = self.read_constant(location).as_object().unwrap().clone();
                    let val = self.peek(0);
                    if self.globals.set(name.clone(), val) {
                        self.globals.delete(&name);
                        self.runtime_error(&format!(
                            "Undefined variable '{}'.",
                            name.as_string().unwrap().0
                        ));
                        return Err(InterpreterError::RuntimeError);
                    }
                }

                GetLocal { stack_idx } => {
                    self.stack.push(self.stack[stack_idx as usize].clone());
                }
                SetLocal { stack_idx } => {
                    self.stack[stack_idx as usize] = self.peek(0);
                }
                JumpIfFalse { offset } => {
                    if is_falsey(self.peek(0)) {
                        self.ip += offset as usize;
                    }
                }
                Jump { offset } => {
                    self.ip += offset as usize;
                }
                Loop { offset } => self.ip -= offset as usize,
            }
        }
    }

    fn read_constant(&self, location: u8) -> Value<'a> {
        return self.chunk.constants[location as usize].clone();
    }

    fn runtime_error(&mut self, msg: &str) {
        writeln!(self.stderr, "{}", msg);

        let instruction = self.ip - 1;
        let line = self.chunk.code[instruction].1;
        writeln!(self.stderr, "[line {}] in script\n", line);
        self.stack.clear();
    }

    fn concatenate(&mut self) {
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        let a = a.as_object().unwrap().as_string().unwrap();
        let b = b.as_object().unwrap().as_string().unwrap();
        let result = b.0.to_string() + &a.0;

        let res = self.allocate_string(result);
        self.stack.push(Value::Object(res));
    }

    fn allocate_string(&mut self, val: String) -> Rc<LoxObject<'a>> {
        let lox_str = LoxObject::new_string(val);
        let entry = self.strings.find_string(&lox_str).clone();
        if let Some(x) = entry {
            return x;
        }
        let str = Rc::new(lox_str);
        self.strings.set(str.clone(), Value::Nil);
        return str;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InterpreterError {
    CompileError,
    RuntimeError,
}

pub type InterpreterResult = Result<(), InterpreterError>;
