// use crate::{compiler::compile, prelude::*};
use crate::{compiler::Compiler, hashtable::HashTable, prelude::*, value::LoxObject};
use smallvec;

const STACK_MAX: usize = 256;

pub struct VM<'a> {
    chunk: Chunk<'a>,

    // Huh, the book says looking by index is slower them
    // looking by index. Why could that be? Due to additions?
    ip: usize,

    stack: smallvec::SmallVec<[Value<'a>; STACK_MAX]>,

    strings: HashTable<'a>,
}

fn is_falsey(value: Value) -> bool {
    match value {
        Value::Nil => true,
        Value::Bool(x) => !x,
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

impl<'a> VM<'a> {
    pub fn new(chunk: Chunk<'a>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: smallvec::SmallVec::new(),
            strings: HashTable::new(),
        }
    }

    pub fn new_with_strings(chunk: Chunk<'a>, strings: HashTable<'a>) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: smallvec::SmallVec::new(),
            strings,
        }
    }

    pub fn interpret(source: &'a str) -> InterpreterResult {
        let strings = HashTable::new();
        let (chunk, strings) = match Compiler::compile(source, strings) {
            Some(x) => x,
            None => return Err(InterpreterError::CompileError),
        };

        let mut vm = Self::new_with_strings(chunk, strings);
        let result = vm.run();
        return result;
    }

    fn peek(&self, distance: usize) -> Value {
        let v = (-1 - distance as i32) as usize;
        return self.stack.get(v % self.stack.len()).cloned().unwrap();
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
            }
        }
    }

    fn runtime_error(&mut self, msg: &str) {
        eprintln!("{}", msg);

        let instruction = self.ip - 1;
        let line = self.chunk.code[instruction].1;
        eprintln!("[line {}] in script\n", line);
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

#[derive(Debug)]
pub enum InterpreterError {
    CompileError,
    RuntimeError,
}

pub type InterpreterResult = Result<(), InterpreterError>;
