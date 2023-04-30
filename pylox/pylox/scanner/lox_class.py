from pylox.scanner.lox_environment import Environment
from pylox.scanner.lox_function import LoxCallable, UserCallable
from pylox.scanner.lexer import Any, Class, List, final, Token
from typing import TYPE_CHECKING

from pylox.scanner.lox_native_models import LoxRuntimeError, LoxValue
if TYPE_CHECKING:
	from pylox.scanner.ast_interpreter import AstInterpreter


from typing import Any, Dict


@final
class LoxInstance(LoxValue):
	def __init__(self, klass: 'LoxClass') -> None:
		self.klass = klass
		self.props: Dict[str, Any] = {}
		env = Environment(self.klass.closure.resolver, self.klass.closure)
		env.define('this', self)
		for method in self.klass.raw.methods:
			self.props[method.name.lexeme] = UserCallable(method, env)
	
	def lox_name(self) -> str:
		return f"<Instance: {self.klass.name()}>"
	
	def get(self, prop: Token):
		if prop.lexeme in self.props:
			return self.props[prop.lexeme]
		raise LoxRuntimeError(prop, "No such property")

	def set(self, prop: Token, value: Any):
		self.props[prop.lexeme] = value
		

@final
class LoxClass(LoxCallable, LoxValue):
	def __init__(self, klass: Class, closure: 'Environment'):
		self.raw = klass
		self.closure = closure

	def name(self) -> str:
		return self.raw.name.lexeme

	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		return LoxInstance(self)

	def arity(self):
		return 0
	
	def lox_name(self) -> str:
		return f"<Class: {self.name()}>"