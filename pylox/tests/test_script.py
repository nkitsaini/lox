from pylox import lox
from pylox.interpreter import Runner
from typing import TYPE_CHECKING, Any
import pytest
from pathlib import Path
if TYPE_CHECKING:
	from pytest_mock import MockerFixture

FIXTURE_PATH = Path(__file__).parent/'fixtures'
SCRIPT_PATH = FIXTURE_PATH/'scripts'

@pytest.fixture(autouse=True)
def run_around_tests():
    lox.reset__()

@pytest.mark.parametrize("script", [x.name for x in SCRIPT_PATH.iterdir()])
def test_script(script: str, mocker: 'MockerFixture', snapshot: Any):
	runner = Runner()
	error_spy = mocker.spy(lox, 'error')
	runtime_spy = mocker.spy(lox, 'runtime_error')
	token_spy = mocker.spy(lox, 'errorToken')
	print_spy = mocker.spy(lox, 'lox_print')
	runner.run((SCRIPT_PATH/script).read_text())
	assert error_spy.call_args_list  == snapshot
	assert runtime_spy.call_args_list  == snapshot
	assert token_spy.call_args_list  == snapshot
	assert print_spy.call_args_list  == snapshot
		