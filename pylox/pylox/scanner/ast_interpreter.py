from typing import Any
from pylox.scanner.lexer import Assignment, Block, If
from .lexer import *
from .tokens import TokenType
from .. import lox


class LoxRuntimeError(Exception):

	def __init__(self, token: Token, message: str) -> None:
		super().__init__(message)
		self.token = token

class _UninitializedVar:
	pass
_UN_INITIALIZED = _UninitializedVar()

@dataclass
class Environment:
	parent: 'Environment | None' = None
	envs: Dict[str, Any] = field(default_factory=dict)

	def define(self, variable: str):
		self.envs[variable] = _UN_INITIALIZED

	def set(self, variable: Token, value: Any) -> Any:
		name = variable.lexeme
		if name in self.envs:
			self.envs[name] = value
			return value
		elif self.parent:
			return self.parent.set(variable, value)
		raise LoxRuntimeError(variable, f"Undefined variable: {name}")
	
	# def is_defined(self, variable: str) -> bool:
	# 	return variable in self.envs

	# def is_initialized(self, variable: str):
	# 	return self.is_defined(variable) and self.envs[variable] is not _UN_INITIALIZED

	def get(self, variable: Token) -> Any:
		name = variable.lexeme
		if name in self.envs:
			if self.envs[name] is _UN_INITIALIZED:
				raise LoxRuntimeError(variable, f"Uninitialized variable: {name}.")
			return self.envs[name]
		elif self.parent:
			return self.parent.get(variable)
		raise LoxRuntimeError(variable, f"Undefined variable: {name}")



@final
class AstInterpreter(ExprVisitor[Any], StmtVisitor[None]):
	def __init__(self) -> None:
		super().__init__()
		self.env: Environment = Environment()
	
	def visit_binary(self, expr: Binary):
		l = self.visit_any(expr.left)
		r = self.visit_any(expr.right)
		match expr.operator.token_type:
			case TokenType.SLASH:
				self.check_number_operands(expr.operator, l, r)
				try:
					return l/r
				except ZeroDivisionError:
					raise LoxRuntimeError(expr.operator, "Can't divide by Zero")
			case TokenType.PLUS:
				if (isinstance(l, str) and isinstance(r, str)):
					return l + r
				elif (isinstance(l, (int, float)) and isinstance(r, (int, float))):
					return l + r
				else:
					raise LoxRuntimeError(expr.operator, "Operands must be two numbers or two strings")
			case TokenType.MINUS:
				self.check_number_operands(expr.operator, l, r)
				return l - r
			case TokenType.STAR:
				self.check_number_operands(expr.operator, l, r)
				return l * r
			case TokenType.GREATER:
				self.check_number_operands(expr.operator, l, r)
				return l > r
			case TokenType.GREATER_EQUAL:
				self.check_number_operands(expr.operator, l, r)
				return l >= r
			case TokenType.LESS:
				self.check_number_operands(expr.operator, l, r)
				return l < r
			case TokenType.LESS_EQUAL:
				self.check_number_operands(expr.operator, l, r)
				return l <= r
			case TokenType.BANG_EQUAL:
				return not self.is_equal(l, r)
			case TokenType.EQUAL_EQUAL:
				return self.is_equal(l, r)
			case _ as op:
				raise RuntimeError(expr.operator, f"Bug in interpreter. Unexpected binary operator. {op}")

	def visit_grouping(self, expr: Grouping):
		return self.visit_any(expr.expression)

	def visit_literal(self, expr: Literal):
		return expr.value
	
	def is_equal(self, a: Any, b: Any):
		return a == b

	def is_truthy(self, val: Any):
		if isinstance(val, bool):
			return val
		elif val == None or val == 0:
			return False
		return True
	
	def check_number_operand(self, operator: Token, operand: Any):
		if isinstance(operand, (int, float)):
			return
		raise LoxRuntimeError(operator, 'Operand must be a number')

	def check_number_operands(self, operator: Token, left: Any, right: Any):
		if isinstance(left, (int, float)) and isinstance(right, (int, float)):
			return
		raise LoxRuntimeError(operator, 'Operands must be a numbers')

	def visit_unary(self, expr: Unary):
		match expr.operator.token_type:
			case TokenType.MINUS:
				e = self.visit_any(expr.right)
				if isinstance(e, (int, float)):
					return e * -1
				else:
					raise LoxRuntimeError(expr.operator, "Can't use - operator on non-numeric objects.")
			case TokenType.BANG:
				e = self.visit_any(expr.right)
				return not self.is_truthy(e)
			case _ as op:
				raise RuntimeError(expr.operator, f"Bug in interpreter. Unexpected binary operator. {op}")

	def interpret(self, statements: List[Statement]):
		try:
			rv = None
			for stmt in statements:
				rv = self.visit_any(stmt)
			return rv
		except LoxRuntimeError as e:
			lox.runtime_error(e)
	
	def visit_any(self, expr: BaseExpr | Statement) -> Any:
		return expr.run_against(self)
			

	def visit_expression(self, expr: Expression) -> Any:
		return expr.expression.run_against(self)
		# return super().visit_expression(expr)
	
	def visit_var(self, expr: Var) -> None:
		# Think if redeclaration should be allowed
		# if expr.name.lexeme in self.globals:
		# 	raise LoxRuntimeError(expr.name, "Variable redclaration")
		name = expr.name.lexeme
		self.env.define(name)
		if expr.expression is not None:
			self.env.set(expr.name, expr.expression.run_against(self))
	
	def visit_assignment(self, expr: Assignment) -> Any:
		return self.env.set(expr.name, expr.expr.run_against(self))

	def visit_variable(self, expr: Variable) -> Any:
		return self.env.get(expr.name)
	
	def printer(self, val: Any):
		# In lox the true and false are lowercase
		if val is True:
			lox.lox_print('true')
		elif val is False:
			lox.lox_print('false')
		else:
			lox.lox_print(val)
	
	def visit_print(self, expr: Print) -> None:
		self.printer(expr.expression.run_against(self))
	
	def visit_block(self, expr: Block) -> None:
		self.env = Environment(self.env)
		try:
			for statement in expr.statements:
				self.visit_any(statement)
		finally:
			assert self.env.parent != None
			self.env = self.env.parent
	
	def visit_if(self, expr: If) -> None:
		if (self.is_truthy(self.visit_any(expr.condition))):
			return self.visit_any(expr.inner)
		elif expr.else_inner is not None:
			return self.visit_any(expr.else_inner)
		

if __name__ == "__main__":
	interpreter = AstInterpreter()
	expr = Binary(
		Unary(Token(TokenType.MINUS, '-', 0, None), Literal(123)),
		Grouping(Literal(54.234)),
		Token(TokenType.STAR, '*', 0, None)
	)
	print(expr.run_against(interpreter))
	print()