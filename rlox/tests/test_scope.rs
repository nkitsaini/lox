use std::io::Cursor;

use pretty_assertions::assert_eq;
use rlox::vm::{InterpreterError, VM};

#[test]
fn test_scope() {
    const SCRIPT: &'static str = "
var a = 3;
a  = a + 2;

var b = 9;

{
	var b = 7;
	b = b + a;
	print b;
}

print a;
print b;
";
    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let mut vm = VM::empty_new(&mut stdout, &mut stderr);
    let res = vm.interpret(SCRIPT);

    drop(vm);

    assert_eq!(res, Ok(()));
    assert!(stderr.into_inner().is_empty());
    let stdout = stdout.into_inner();
    let out = String::from_utf8(stdout).unwrap();
    insta::assert_snapshot!(out);
}

#[test]
fn test_scope_error() {
    const SCRIPT: &'static str = "
var a = 3;
a  = a + 2;

var b = 9;

{
	var b = b; // this should show error
}

print a;
print b;
";
    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let mut vm = VM::empty_new(&mut stdout, &mut stderr);
    let res = vm.interpret(SCRIPT);

    drop(vm);

    assert_eq!(res, Err(InterpreterError::CompileError));
    insta::assert_snapshot!(String::from_utf8(stderr.into_inner()).unwrap());
    insta::assert_snapshot!(String::from_utf8(stdout.into_inner()).unwrap());
}
