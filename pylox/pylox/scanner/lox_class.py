from pylox.scanner.lox_function import LoxCallable
from pylox.scanner.lexer import Any, Class, List, final
from typing import TYPE_CHECKING

from pylox.scanner.lox_native_models import LoxValue
if TYPE_CHECKING:
	from pylox.scanner.ast_interpreter import AstInterpreter


from typing import Any

@final
class LoxInstance(LoxValue):
	def __init__(self, klass: 'LoxClass') -> None:
		self.klass = klass
	
	def lox_name(self) -> str:
		return f"<Instance: {self.klass.name()}>"

@final
class LoxClass(LoxCallable, LoxValue):
	def __init__(self, klass: Class):
		self.raw = klass

	def name(self) -> str:
		return self.raw.name.lexeme

	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		return LoxInstance(self)

	def arity(self):
		return 0
	
	def lox_name(self) -> str:
		return f"<Class: {self.name()}>"