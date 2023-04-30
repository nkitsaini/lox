import time
from pylox.scanner.lox_environment import Environment
from pylox.scanner.lexer import AnonFunction, Any, Function, List, abc, final
from pylox.scanner.lox_native_models import FunctionReturn
from .lox_native_models import LoxValue

from typing import TYPE_CHECKING

if TYPE_CHECKING:
	from pylox.scanner.ast_interpreter import AstInterpreter

from typing import Any


class LoxCallable(abc.ABC, LoxValue):
	@abc.abstractmethod
	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		raise NotImplementedError()

	@abc.abstractmethod
	def arity(self) -> int:
		raise NotImplementedError()

	@abc.abstractmethod
	def name(self) -> str:
		raise NotImplementedError()
	


@final
class UserCallable(LoxCallable):
	def __init__(self, fn: Function, closure: 'Environment') -> None:
		self.fn = fn
		self.closure = closure

	def name(self) -> str:
		return self.fn.name.lexeme

	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		new_env = Environment(interpreter.resolver)
		for lexeme, val_expr in zip(self.fn.arguments, arguments):
			new_env.define(lexeme.lexeme, val_expr)

		old_env = interpreter.env
		new_env.parent = self.closure
		interpreter.env = new_env

		try:
			interpreter.visit_any(self.fn.body)
		except FunctionReturn as r:
			return r.return_val
		finally:
			interpreter.env = old_env

	def arity(self) -> int:
		return len(self.fn.arguments)

	def lox_name(self) -> str:
		return f"<function: {self.name()}>"

@final
class AnonymousUserCallable(LoxCallable):
	def __init__(self, fn: AnonFunction, closure: 'Environment') -> None:
		self.fn = fn
		self.closure = closure

	def name(self) -> str:
		return ":Anonymous:"

	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		new_env = Environment(interpreter.resolver)
		for lexeme, val_expr in zip(self.fn.arguments, arguments):
			new_env.define(lexeme.lexeme, val_expr)

		old_env = interpreter.env
		new_env.parent = self.closure
		interpreter.env = new_env

		try:
			interpreter.visit_any(self.fn.body)
		except FunctionReturn as r:
			return r.return_val
		finally:
			interpreter.env = old_env

	def arity(self) -> int:
		return len(self.fn.arguments)

	def lox_name(self) -> str:
		return f"<AnonymousUserFunction>"


class NativeLoxCallable(LoxCallable):
	@staticmethod
	@abc.abstractmethod
	def static_name() -> str:
		raise NotImplementedError()

	def name(self) -> str:
		return self.static_name()

	def lox_name(self) -> str:
		return f"<NativeLoxFunction: {self.name()}>"

@final
class ClockCallable(NativeLoxCallable):
	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		return time.time()

	def arity(self) -> int:
		return 0

	@staticmethod
	def static_name() -> str:
		return "clock"