use std::io::Cursor;

use pretty_assertions::assert_eq;
use rlox::vm::{InterpreterError, InterpreterResult, VM};

#[inline]
pub fn test_execution(response_type: InterpreterResult, source: &str, name: &str) {
    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let mut vm = VM::empty_new(&mut stdout, &mut stderr);
    let res = vm.interpret(source);

    drop(vm);

    assert_eq!(res, response_type);
    insta::assert_snapshot!(name, String::from_utf8(stderr.into_inner()).unwrap());
    insta::assert_snapshot!(name, String::from_utf8(stdout.into_inner()).unwrap());
}

#[inline]
pub fn test_successfull_execution(source: &str, name: &str) {
    test_execution(Ok(()), source, name);
}
