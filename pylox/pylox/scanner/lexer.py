from .tokens import *
import abc
from ..prelude import *


_VisitorReturn = TypeVar("_VisitorReturn")

class BaseExpr(abc.ABC):
	@abc.abstractmethod
	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		raise NotImplementedError()


@final
@dataclass
class Binary(BaseExpr):
	left: BaseExpr
	right: BaseExpr
	operator: Token

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_binary(self)

@final
@dataclass
class Grouping(BaseExpr):
	expression: BaseExpr

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_grouping(self)

@final
@dataclass
class Literal(BaseExpr):
	value: Any

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_literal(self)

@final
@dataclass
class Unary(BaseExpr):
	operator: Token
	right: BaseExpr

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_unary(self)

@final
@dataclass
class Variable(BaseExpr):
	name: Token

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_variable(self)

@final
@dataclass
class Assignment(BaseExpr):
	name: Token
	expr: BaseExpr

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_assignment(self)

@final
@dataclass
class Logical(BaseExpr):
	left: BaseExpr
	operator: Token
	right: BaseExpr

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_logical(self)

@final
@dataclass
class Call(BaseExpr):
	callee: BaseExpr
	paren: Token
	arguments: List[BaseExpr]

	def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_call(self)


class ExprVisitor(abc.ABC, Generic[_VisitorReturn]):
	@abc.abstractmethod
	def visit_binary(self, expr: 'Binary') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_grouping(self, expr: 'Grouping') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_literal(self, expr: 'Literal') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_unary(self, expr: 'Unary') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_variable(self, expr: 'Variable') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_assignment(self, expr: 'Assignment') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_logical(self, expr: 'Logical') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_call(self, expr: 'Call') -> _VisitorReturn:
		raise NotImplementedError()

	def visit_any(self, expr: BaseExpr) -> _VisitorReturn:
		return expr.run_against(self)

class Statement(abc.ABC):
	@abc.abstractmethod
	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		raise NotImplementedError()

@final
@dataclass
class Expression(Statement):
	expression: BaseExpr

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_expression(self)

@final
@dataclass
class Print(Statement):
	expression: BaseExpr

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_print(self)

@final
@dataclass
class Var(Statement):
	expression: Optional[BaseExpr]
	name: Token

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_var(self)

@final
@dataclass
class Block(Statement):
	statements: List[Statement]

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_block(self)

@final
@dataclass
class If(Statement):
	condition: BaseExpr
	inner: Statement
	else_inner: Optional[Statement]

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_if(self)

@final
@dataclass
class While(Statement):
	condition: BaseExpr
	inner: Statement

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_while(self)

@final
@dataclass
class Break(Statement):
	pass
	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_break(self)

@final
@dataclass
class Return(Statement):
	expr: BaseExpr

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_return(self)

@final
@dataclass
class Function(Statement):
	name: Token
	arguments: List[Token]
	body: Statement

	def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_function(self)


class StmtVisitor(abc.ABC, Generic[_VisitorReturn]):
	@abc.abstractmethod
	def visit_expression(self, expr: 'Expression') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_print(self, expr: 'Print') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_var(self, expr: 'Var') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_block(self, expr: 'Block') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_if(self, expr: 'If') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_while(self, expr: 'While') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_break(self, expr: 'Break') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_return(self, expr: 'Return') -> _VisitorReturn:
		raise NotImplementedError()

	@abc.abstractmethod
	def visit_function(self, expr: 'Function') -> _VisitorReturn:
		raise NotImplementedError()



