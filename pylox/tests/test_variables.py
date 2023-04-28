

from pylox import lox
from pylox.interpreter import Runner
from typing import TYPE_CHECKING, Any
import pytest
if TYPE_CHECKING:
	from pytest_mock import MockerFixture

variables = """
	var a;
	var b;
	var c;
	print a;
	print b;
	print c;
	print d;
	a=b=3;
	print a;
	print b;
	print c;
	print d;
	a=b+1;
	print a;
	print b;
	print c;
	print d;
	print d
	a=3
"""

scoping = """
var a = "global a";
var b = "global b";
var c = "global c";
{
  var a = "outer a";
  var b = "outer b";
  {
    var a = "inner a";
    print a;
    print b;
    print c;
  }
  print a;
  print b;
  print c;
}
print a;
print b;
print c;
"""

if_conditional = """
var a =3;
var b =7;
if (true) {
	print a;
} else {
	print b;
}
if (false) {
	print a;
} else {
	print b;
}

if (true) {
	print b;
}
if (1) {
	print b;
}
if (false) {
	print b;
}

"""

boolean_control_flow = """
true or false;
false or true;
true or true;
false or false;
true and false;
false and true;
true and true;
false and false;
true and 2;
2 and true;
true and true;
2 and false;
1 and 2;
2 and 1;
1 and 1;
2 and false;
"""

@pytest.fixture(autouse=True)
def run_around_tests():
    lox.reset__()

@pytest.mark.parametrize("script", [variables, boolean_control_flow])
def test_repl(script: str, mocker: 'MockerFixture', snapshot: Any):
	runner = Runner()
	error_spy = mocker.spy(lox, 'error')
	runtime_spy = mocker.spy(lox, 'runtime_error')
	token_spy = mocker.spy(lox, 'errorToken')
	print_spy = mocker.spy(lox, 'lox_print')
	for statement in script.splitlines():
		print(statement)
		runner.run(statement)
		assert error_spy.call_args_list  == snapshot
		assert runtime_spy.call_args_list  == snapshot
		assert token_spy.call_args_list  == snapshot
		assert print_spy.call_args_list  == snapshot
		
@pytest.mark.parametrize("script", [scoping, if_conditional])
def test_script(script: str, mocker: 'MockerFixture', snapshot: Any):
	runner = Runner()
	error_spy = mocker.spy(lox, 'error')
	runtime_spy = mocker.spy(lox, 'runtime_error')
	token_spy = mocker.spy(lox, 'errorToken')
	print_spy = mocker.spy(lox, 'lox_print')
	runner.run(script)
	assert error_spy.call_args_list  == snapshot
	assert runtime_spy.call_args_list  == snapshot
	assert token_spy.call_args_list  == snapshot
	assert print_spy.call_args_list  == snapshot
		