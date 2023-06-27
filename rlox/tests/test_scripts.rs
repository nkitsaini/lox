#[macro_use]
mod common;

use rlox::vm::InterpreterError;

#[test]
fn test_scope() {
    test_execution_success!("./fixtures/scope.lox");
}

#[test]
fn test_scope_error() {
    test_execution!(
        Err(InterpreterError::CompileError),
        "fixtures/scope_error.lox"
    );
}

#[test]
fn test_if() {
    test_execution_success!("fixtures/if.lox");
}
#[test]
fn test_if_else() {
    test_execution_success!("fixtures/if_else.lox");
}

#[test]
fn test_or_and() {
    test_execution_success!("fixtures/or_and.lox");
}
#[test]
fn test_while() {
    test_execution_success!("fixtures/while.lox");
}
#[test]
fn test_for() {
    test_execution_success!("fixtures/for.lox");
}
