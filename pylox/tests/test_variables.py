

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

@pytest.fixture(autouse=True)
def run_around_tests():
    lox.reset__()

@pytest.mark.parametrize("script", [variables])
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
		
@pytest.mark.parametrize("script", [scoping])
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
		