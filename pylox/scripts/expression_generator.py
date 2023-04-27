import textwrap

def main():
	TAB = "\t"
	expressions = {
		"Binary": "left BaseExpr, right BaseExpr, operator Token",
		"Grouping": "expression BaseExpr",
		"Literal": "value Any",
		"Unary": "operator Token, right BaseExpr",
		"Variable": "name Token",
		"Assignment": "name Token, expr BaseExpr ",
	}

	statements = {
		"Expression": "expression BaseExpr",
		"Print": "expression BaseExpr",
		"Var": "expression Optional[BaseExpr], name Token",
	}

	header = textwrap.dedent("""\
		from .tokens import *
		import abc
		from ..prelude import *


	""")

	out = header

	### Create classes

	## Base
	out += '_VisitorReturn = TypeVar("_VisitorReturn")\n'
	out += '\n'

	out += "class BaseExpr(abc.ABC):\n"

	out += TAB + "@abc.abstractmethod\n"
	out += TAB + "def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:\n"
	out += TAB + TAB +  "raise NotImplementedError()\n"
	out += "\n"
	# for expression_name in expressions.keys():
	# 	out += TAB + "@abc.abstractmethod\n"
	# 	out += TAB + "def visit_" + expression_name.lower() + f"(self, expr: '{expression_name}'):\n"
	# 	out += TAB + TAB +  "raise NotImplementedError()\n"
	# 	out += "\n"
	out += "\n"

	## Rest
	for expression_name, fields in expressions.items():
		out += f"@final\n"
		out += f"@dataclass\n"
		out += f"class {expression_name}(BaseExpr):\n"
		for field_info in fields.split(','):
			field_name, field_type = field_info.strip().split(' ')
			out += TAB + f"{field_name}: {field_type}\n"
		out += "\n"
		out += TAB + "def run_against(self, visitor: 'ExprVisitor[_VisitorReturn]') -> _VisitorReturn:\n"
		out += TAB + TAB +  "return visitor.visit_" + expression_name.lower() + "(self)\n"
		out += "\n"
	# out += "Expression = " + "|".join(expressions.keys()) + "\n"
	### Visitor

	out += "\n"

	out += "class ExprVisitor(abc.ABC, Generic[_VisitorReturn]):\n"

	# out += TAB + "@abc.abstractmethod\n"
	# out += TAB + "def visit(self, expr: '{expression_name}'):\n"
	# out += TAB + TAB +  "raise NotImplementedError()\n"
	# out += "\n"
	for expression_name in expressions.keys():
		out += TAB + "@abc.abstractmethod\n"
		out += TAB + "def visit_" + expression_name.lower() + f"(self, expr: '{expression_name}') -> _VisitorReturn:\n"
		out += TAB + TAB +  "raise NotImplementedError()\n"
		out += "\n"

	out += TAB + f"def visit_any(self, expr: BaseExpr) -> _VisitorReturn:\n"
	out += TAB + TAB +  "return expr.run_against(self)"
	out += "\n"
	out += "\n"

	## Base


	###	Statemetns
	out += "class Statement(abc.ABC):\n"

	out += TAB + "@abc.abstractmethod\n"
	out += TAB + "def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:\n"
	out += TAB + TAB +  "raise NotImplementedError()\n"
	out += "\n"
	## Rest
	for expression_name, fields in statements.items():
		out += f"@final\n"
		out += f"@dataclass\n"
		out += f"class {expression_name}(Statement):\n"
		for field_info in fields.split(','):
			field_name, field_type = field_info.strip().split(' ')
			out += TAB + f"{field_name}: {field_type}\n"
		out += "\n"
		out += TAB + "def run_against(self, visitor: 'StmtVisitor[_VisitorReturn]') -> _VisitorReturn:\n"
		out += TAB + TAB +  "return visitor.visit_" + expression_name.lower() + "(self)\n"
		out += "\n"


	out += "\n"

	out += "class StmtVisitor(abc.ABC, Generic[_VisitorReturn]):\n"

	# out += TAB + "@abc.abstractmethod\n"
	# out += TAB + "def visit(self, expr: '{expression_name}'):\n"
	# out += TAB + TAB +  "raise NotImplementedError()\n"
	# out += "\n"
	for expression_name in statements.keys():
		out += TAB + "@abc.abstractmethod\n"
		out += TAB + "def visit_" + expression_name.lower() + f"(self, expr: '{expression_name}') -> _VisitorReturn:\n"
		out += TAB + TAB +  "raise NotImplementedError()\n"
		out += "\n"

	# out += TAB + f"def visit_any(self, expr: Statement) -> _VisitorReturn:\n"
	# out += TAB + TAB +  "return expr.run_against(self)"
	out += "\n"
	out += "\n"



	print(out)

if __name__ == "__main__":
	main()