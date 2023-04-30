from pylox.scanner.lox_native_models import LoxRuntimeError
from pylox.scanner.lexer import Any, Dict, Optional, Token, dataclass, field
from pylox.scanner.resolver import AstResolver


from typing import Any


class _UninitializedVar:
	pass
_UN_INITIALIZED = _UninitializedVar()


@dataclass
class Environment:
	resolver: 'AstResolver'
	parent: 'Environment | None' = None
	envs: Dict[str, Any] = field(default_factory=dict)

	def declare(self, variable: str):
		self.envs[variable] = _UN_INITIALIZED

	def define(self, variable: str, value: Any):
		self.envs[variable] = value
	def set(self, variable: Token, value: Any, depth: Optional[int] = None) -> Any:
		if depth == None:
			depth = self.resolver.variable_to_depth[variable]
		name = variable.lexeme
		if depth == 0:
			self.envs[name] = value
			return value
		assert self.parent, "Compiler bug, parent undefined but didn't catch in resolver"
		return self.parent.set(variable, value, depth - 1)

	def get_by_str(self, variable: str, depth: int) -> Any:
		name = variable
		if depth == 0:
			if self.envs[name] is _UN_INITIALIZED:
				raise RuntimeError("Can't use get_by_str for possibly non-initialized stuff")

			return self.envs[name]
		assert self.parent, "Compiler bug, parent undefined but didn't catch in resolver. get"
		return self.parent.get_by_str(variable, depth-1)
	
	def get_depth(self, variable: Token) -> int:
		return self.resolver.variable_to_depth[variable]

	def get(self, variable: Token, depth: Optional[int] = None) -> Any:
		if depth == None:
			depth = self.resolver.variable_to_depth[variable]
		name = variable.lexeme
		if depth == 0:
			if self.envs[name] is _UN_INITIALIZED:
				raise LoxRuntimeError(variable, f"Uninitialized variable: {name}.")

			return self.envs[name]
		assert self.parent, "Compiler bug, parent undefined but didn't catch in resolver. get"
		return self.parent.get(variable, depth-1)