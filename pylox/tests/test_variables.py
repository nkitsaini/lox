

from pylox import lox
from pylox.interpreter import Runner
from typing import TYPE_CHECKING
if TYPE_CHECKING:
	from pytest_mock import MockerFixture

def test_variable_assignments(mocker: 'MockerFixture', snapshot):
	runner = Runner()
	script = """
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
	error_spy = mocker.spy(lox, 'error')
	runtime_spy = mocker.spy(lox, 'runtime_error')
	token_spy = mocker.spy(lox, 'errorToken')
	print_spy = mocker.spy(lox, 'lox_print')
	for statement in script.splitlines():
		runner.run(statement)
		assert error_spy.call_args_list  == snapshot
		assert runtime_spy.call_args_list  == snapshot
		assert token_spy.call_args_list  == snapshot
		assert print_spy.call_args_list  == snapshot