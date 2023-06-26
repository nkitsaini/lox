use std::io::Cursor;

use pretty_assertions::assert_eq;
use rlox::vm::VM;

const script: &'static str = "
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

#[test]
fn test_scope() {
    let mut stdout = Cursor::new(Vec::new());
    let mut stderr = Cursor::new(Vec::new());
    let mut vm = VM::empty_new(&mut stdout, &mut stderr);
    let res = vm.interpret(script);
    drop(vm);

    assert_eq!(res, Ok(()));
    assert!(stderr.into_inner().is_empty());
    let stdout = stdout.into_inner();
    let out = String::from_utf8(stdout).unwrap();
    insta::assert_snapshot!(out);
}
