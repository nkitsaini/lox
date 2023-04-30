
from typing import Any, Protocol, runtime_checkable
import abc

from pylox.scanner.lexer import Any, Token

@runtime_checkable
class LoxValue(Protocol):
	@abc.abstractmethod
	def lox_name(self) -> str:
		raise NotImplementedError()


class LoxRuntimeError(Exception):

	def __init__(self, token: Token, message: str) -> None:
		super().__init__(message)
		self.token = token


class FunctionReturn(Exception):
	def __init__(self, return_val: Any) -> None:
		super().__init__("")
		self.return_val = return_val


class LoopBreak(Exception):
	pass


