from pylox import lox
from pylox.interpreter import Runner
from typing import TYPE_CHECKING, Any
import pytest
from pathlib import Path
if TYPE_CHECKING:
	from pytest_mock import MockerFixture
FIXTURE_PATH = Path(__file__).parent/'fixtures'
REPL_PATH = FIXTURE_PATH/'repls'

@pytest.fixture(autouse=True)
def run_around_tests():
    lox.reset__()

@pytest.mark.parametrize("script", [x.name for x in REPL_PATH.iterdir()])
def test_repl(script: str, mocker: 'MockerFixture', snapshot: Any):
	runner = Runner()
	error_spy = mocker.spy(lox, 'error')
	runtime_spy = mocker.spy(lox, 'runtime_error')
	token_spy = mocker.spy(lox, 'errorToken')
	print_spy = mocker.spy(lox, 'lox_print')

	for statement in (REPL_PATH/script).read_text().splitlines():
		print(statement)
		runner.run(statement)
		assert error_spy.call_args_list  == snapshot
		assert runtime_spy.call_args_list  == snapshot
		assert token_spy.call_args_list  == snapshot
		assert print_spy.call_args_list  == snapshot
		