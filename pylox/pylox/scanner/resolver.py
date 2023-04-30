from pylox.scanner.lexer import Assignment, Binary, Block, Expression, Function, Grouping, If, Print, Var, Variable, While
from contextlib import contextmanager
from .lexer import *
from .. import lox

@final
class AstResolver(ExprVisitor[None], StmtVisitor[None]):
	def __init__(self) -> None:
		# Global/nested/nested
		self.scopes: List[Dict[str, bool]] = [{"clock": True}] # [{variable_name: is_initialized}]
		self.open_loops = 0
		self.open_functions = 0
		self.variable_to_depth: Dict[Token, int] = {} # identifier: int

	def resolve(self, statements: 'List[Statement] | Statement | BaseExpr'):
		if isinstance(statements, list):
			for statement in statements:
				self.resolve(statement)
		else:
			statements.run_against(self)
	
	@contextmanager
	def new_scope(self):
		self.scopes.append({})
		yield
		self.scopes.pop()

	def visit_block(self, expr: Block) -> None:
		with self.new_scope():
			self.resolve(expr.statements)
	
	def visit_var(self, expr: Var) -> None:
		self.declare(expr.name)
		if expr.expression is not None:
			self.resolve(expr.expression)
			self.define(expr.name)
	
	def visit_variable(self, expr: Variable) -> None:
		self.pin_resolution(expr.name)
	
	def visit_assignment(self, expr: Assignment) -> None:
		self.resolve(expr.expr)
		self.pin_definition(expr.name)
	
	def visit_function(self, expr: Function) -> None:
		self.define(expr.name)
		with self.new_scope():
			for arg in expr.arguments:
				self.define(arg)
			self.resolve(expr.body)
	
	def visit_class(self, expr: Class):
		self.define(expr.name)
		# TODO: maybe need to do something with methods here?
	
	def visit_expression(self, expr: Expression) -> None:
		self.resolve(expr.expression)
		
	def visit_if(self, expr: If) -> None:
		self.resolve(expr.condition)
		# we don't need to create new scope here, since
		# a new block_statement will automatically be created if the code uses block statement.
		# Otherwise the parent scope will get used.
		self.resolve(expr.inner)
		if expr.else_inner is not None:
			self.resolve(expr.else_inner)
	
	def visit_print(self, expr: Print) -> None:
		self.resolve(expr.expression)

	def visit_return(self, expr: Return) -> None:
		if expr.expr is not None:
			self.resolve(expr.expr)
	
	def visit_while(self, expr: While) -> None:
		self.resolve(expr.condition)
		self.resolve(expr.inner)
	
	def visit_binary(self, expr: Binary) -> None:
		self.resolve(expr.left)
		self.resolve(expr.right)

	def visit_call(self, expr: Call) -> None:
		self.resolve(expr.callee)
		for arg in expr.arguments:
			self.resolve(arg)
	
	def visit_grouping(self, expr: Grouping) -> None:
		self.resolve(expr.expression)
	
	def visit_literal(self, expr: Literal):
		pass

	def visit_logical(self, expr: Logical):
		self.resolve(expr.left)
		self.resolve(expr.right)

	def visit_unary(self, expr: Unary):
		self.resolve(expr.right)

	def pin_definition(self, name: Token):
		"""
		Definition is invalid if we never find a variable at all

		Straying away from lox definition
		It's allowed to shadow an outer variable
		```
		var a= 1;
		{
			var a = a; // lox does not allow, but we'll
		}
		```
		"""
		for (depth, scope) in enumerate(self.scopes[::-1]):
			if name.lexeme in scope:
				self.variable_to_depth[name] = depth
				return
		lox.errorToken(name, "Unknown variable usage")

	def pin_resolution(self, name: Token):
		"""
		Resolution is invalid if we find a declared variable before a defined one.
		Or we don't find variable at all.

		Straying away from lox definition
		It's allowed to shadow an outer variable
		```
		var a= 1;
		{
			var a = a; // lox does not allow, but we'll
		}
		```
		"""
		for (depth, scope) in enumerate(self.scopes[::-1]):
			if name.lexeme in scope:
				if scope[name.lexeme] == False:
					# TODO: make it more narrow and re-enable
					# lox.warningToken(name, "Possibly Uninitialized variable usage")
					pass
				self.variable_to_depth[name] = depth
				return
		lox.errorToken(name, "Unknown variable usage")

	def declare(self, name: Token):
		# If similar variable is defined in the scope already, don't override it.
		# Make it a warning?
		if self.scopes[-1].get(name.lexeme) == True:
			lox.warningToken(name, "Variable is already declared in this scope. Did you want to reuse that?")
			return
		self.scopes[-1][name.lexeme] = False

	def define(self, name: Token):
		self.scopes[-1][name.lexeme] = True
	
	def visit_break(self, expr: Break):
		return

	def visit_anonfunction(self, expr: AnonFunction):
		with self.new_scope():
			for arg in expr.arguments:
				self.define(arg)
			self.resolve(expr.body)

