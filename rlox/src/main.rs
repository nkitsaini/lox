#![feature(type_alias_impl_trait)]
use std::fs;
use std::io;
use std::io::Write;
mod chunk;
mod compiler;
mod debug;
mod hashtable;
mod prelude;
mod scanner;
mod value;
mod vm;

use vm::VM;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1])?;
    } else {
        eprintln!("Usage: clox [path]");
        std::process::exit(64);
    }

    Ok(())
}

fn repl() {
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush().ok();
    let mut out = io::stdout();
    let mut err = io::stderr();
    let mut vm = VM::empty_new(&mut out, &mut err);
    for line in stdin.lines() {
        match line {
            Err(_) => {
                eprintln!("[REPL Error]: Invalid input, try again!");
            }
            Ok(x) => {
                // TODO: We make it static to simplify lifetime of
                // tokens and LoxObjects
                // LoxObjects now allocate their own string, so only tokens are remaining
                // deal with it and remove the leak
                let x: &'static str = Box::leak(Box::new(x));
                vm.interpret(&x).ok();
            }
        };
        print!("> ");
        io::stdout().flush().ok();
    }
    println!();
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(path)?;

    let mut out = io::stdout();
    let mut err = io::stderr();
    let mut vm = VM::empty_new(&mut out, &mut err);
    let result = vm.interpret(&source);

    let error = match result {
        Err(x) => x,
        Ok(_) => return Ok(()),
    };

    match error {
        vm::InterpreterError::CompileError => {
            std::process::exit(65);
        }
        vm::InterpreterError::RuntimeError => {
            std::process::exit(70);
        }
    };
}
