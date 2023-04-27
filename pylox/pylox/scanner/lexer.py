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




