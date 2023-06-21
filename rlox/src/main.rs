#![feature(type_alias_impl_trait)]
use std::fs;
use std::io;
use std::io::Write;
mod chunk;
mod compiler;
mod debug;
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
    for line in stdin.lines() {
        match line {
            Err(_) => {
                eprintln!("[REPL Error]: Invalid input, try again!");
            }
            Ok(x) => {
                VM::interpret(&x).ok();
            }
        };
        print!("> ");
        io::stdout().flush().ok();
    }
    println!();
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(path)?;
    let result = VM::interpret(&source);

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
