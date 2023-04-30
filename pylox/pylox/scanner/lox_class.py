from pylox.scanner.lox_environment import Environment
from pylox.scanner.lox_function import LoxCallable, UserCallable
from pylox.scanner.lexer import Any, Class, Function, List, final, Token, Optional
from typing import TYPE_CHECKING

from pylox.scanner.lox_native_models import LoxRuntimeError, LoxValue
if TYPE_CHECKING:
	from pylox.scanner.ast_interpreter import AstInterpreter


from typing import Any, Dict


@final
class LoxInstance(LoxValue):
	def __init__(self, klass: 'LoxClass', closure: 'Environment') -> None:
		self.klass = klass
		self.props: Dict[str, Any] = {}
		self.env = Environment(closure.resolver, closure)
		self.env.define('this', self)
		for method in self.klass.get_methods():
			self.props[method.name.lexeme] = UserCallable(method, self.env)
	
	def lox_name(self) -> str:
		return f"<Instance: {self.klass.name()}>"
	
	def get(self, prop: Token):
		if prop.lexeme in self.props:
			return self.props[prop.lexeme]
		raise LoxRuntimeError(prop, f"No such property: {prop.lexeme}")

	def set(self, prop: Token, value: Any):
		self.props[prop.lexeme] = value
		

@final
class LoxClass(LoxCallable, LoxValue):
	def __init__(self, klass: Class, closure: 'Environment', superclass: Optional['LoxClass']):
		self.raw = klass
		self.closure = closure
		self.superclass = superclass
		self.env = Environment(closure.resolver, closure)
		self.env.define('super', superclass)
		self.methods_by_names: Dict[str, Function] = {}
		for method in self.get_methods(): # I know it's slow. But I just want to get it to work ;(
			self.methods_by_names[method.name.lexeme] = method

	def name(self) -> str:
		return self.raw.name.lexeme
	
	def get_methods(self) -> List[Function]:
		rv: List[Function] = []
		if self.superclass:
			rv = self.superclass.get_methods()
		rv.extend(self.raw.methods)
		return rv


	def call(self, interpreter: 'AstInterpreter', arguments: List[Any]) -> Any:
		return LoxInstance(self, self.env)

	def arity(self):
		return 0
	
	def lox_name(self) -> str:
		return f"<Class: {self.name()}>"