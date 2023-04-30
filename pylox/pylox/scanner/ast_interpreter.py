from typing import Any
from pylox.scanner.lox_environment import Environment
from pylox.scanner.lox_function import AnonymousUserCallable, ClockCallable, LoxCallable, UserCallable
from pylox.scanner.lexer import Any, Assignment, Block, If, List
from pylox.scanner.lox_class import LoxClass
from pylox.scanner.lox_native_models import FunctionReturn, LoopBreak, LoxRuntimeError, LoxValue
from .lexer import *
from .tokens import TokenType
from .. import lox
from .resolver import AstResolver


@final
class AstInterpreter(ExprVisitor[Any], StmtVisitor[None]):
	def __init__(self, resolver: 'AstResolver') -> None:
		# super().__init__()
		self.resolver = resolver
		self.global_env = Environment(self.resolver)
		self.global_env.define(ClockCallable.static_name(), ClockCallable())
		self.env: Environment = self.global_env
	
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

	def visit_call(self, expr: Call):
		function_ref = self.visit_any(expr.callee)
		if not isinstance(function_ref, LoxCallable):
			raise LoxRuntimeError(expr.paren, "Value is not a callable: " + str(function_ref))
		if function_ref.arity() != len(expr.arguments):
			raise LoxRuntimeError(expr.paren, f"Expected {function_ref.arity()} arguments to be passed. Found {len(expr.arguments)}")

		return function_ref.call(self, [self.visit_any(ex) for ex in expr.arguments])

	def visit_anonfunction(self, expr: AnonFunction):
		return AnonymousUserCallable(expr, self.env)

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

	def visit_logical(self, expr: Logical):
		left_value = self.visit_any(expr.left)
		rv = left_value
		if expr.operator.token_type == TokenType.OR:
			if not self.is_truthy(left_value):
				rv = self.visit_any(expr.right)
		elif expr.operator.token_type == TokenType.AND:
			if self.is_truthy(left_value):
				rv = self.visit_any(expr.right)

		else:
			raise RuntimeError("Compiler Bug")
		return rv
		
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
		result = None
		if expr.expression is not None:
			result = expr.expression.run_against(self)
		self.env.declare(name)
		if expr.expression is not None:
			self.env.define(expr.name.lexeme, result)
	
	def visit_assignment(self, expr: Assignment) -> Any:
		val = expr.expr.run_against(self)
		return self.env.set(expr.name, val)

	def visit_variable(self, expr: Variable) -> Any:
		return self.env.get(expr.name)
	
	def printer(self, val: Any):
		if val is True:
			lox.lox_print('true')
		elif val is False:
			lox.lox_print('false')
		elif val is None:
			lox.lox_print('nil')
		elif isinstance(val, LoxValue):
			lox.lox_print(val.lox_name())
		else:
			lox.lox_print(val)
	
	def visit_print(self, expr: Print) -> None:
		self.printer(expr.expression.run_against(self))
	
	def visit_block(self, expr: Block) -> None:
		self.env = Environment(self.resolver, self.env)
		try:
			for statement in expr.statements:
				self.visit_any(statement)
		finally:
			assert self.env.parent != None
			self.env = self.env.parent
	
	def visit_if(self, expr: If):
		if (self.is_truthy(self.visit_any(expr.condition))):
			return self.visit_any(expr.inner)
		elif expr.else_inner is not None:
			return self.visit_any(expr.else_inner)
		
	def visit_while(self, expr: While) -> None:
		while self.visit_any(expr.condition):
			try:
				self.visit_any(expr.inner)
			except LoopBreak:
				break

	def visit_function(self, expr: Function):
		self.env.declare(expr.name.lexeme)
		self.env.define(expr.name.lexeme, UserCallable(expr, self.env))
	
	def visit_class(self, expr: Class):
		self.env.declare(expr.name.lexeme)
		klass = LoxClass(expr)
		self.env.define(expr.name.lexeme, klass)

	
	def visit_break(self, expr: Break):
		raise LoopBreak()

	def visit_return(self, expr: Return):
		if expr.expr is None:
			raise FunctionReturn(None)
		raise FunctionReturn(self.visit_any(expr.expr))
	


if __name__ == "__main__":
	expr = Binary(
		Unary(Token(TokenType.MINUS, '-', 0, None), Literal(123)),
		Grouping(Literal(54.234)),
		Token(TokenType.STAR, '*', 0, None)
	)
	interpreter = AstInterpreter(AstResolver())
	interpreter.resolver.resolve(expr)
	print(expr.run_against(interpreter))
	print()