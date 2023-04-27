from .tokens import *
import abc
from ..prelude import *


_VisitorReturn = TypeVar("_VisitorReturn")

class BaseExpr(abc.ABC):
	@abc.abstractmethod
	def run_against(self, visitor: 'Visitor[_VisitorReturn]') -> _VisitorReturn:
		raise NotImplementedError()


@final
@dataclass
class Binary(BaseExpr):
	left: BaseExpr
	right: BaseExpr
	operator: Token

	def run_against(self, visitor: 'Visitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_binary(self)

@final
@dataclass
class Grouping(BaseExpr):
	expression: BaseExpr

	def run_against(self, visitor: 'Visitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_grouping(self)

@final
@dataclass
class Literal(BaseExpr):
	value: Any

	def run_against(self, visitor: 'Visitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_literal(self)

@final
@dataclass
class Unary(BaseExpr):
	operator: Token
	right: BaseExpr

	def run_against(self, visitor: 'Visitor[_VisitorReturn]') -> _VisitorReturn:
		return visitor.visit_unary(self)

Expression = Binary|Grouping|Literal|Unary

class Visitor(abc.ABC, Generic[_VisitorReturn]):
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

	def visit_any(self, expr: BaseExpr) -> _VisitorReturn:
		return expr.run_against(self)


