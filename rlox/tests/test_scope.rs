#[macro_use]
mod common;

use rlox::vm::InterpreterError;

#[test]
fn test_scope() {
    let script = "
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

    test_execution_success!(script);
}

#[test]
fn test_scope_error() {
    let script = "
var a = 3;
a  = a + 2;

var b = 9;

{
	var b = b; // this should show error
}

print a;
print b;
";

    test_execution!(Err(InterpreterError::CompileError), script);
}
