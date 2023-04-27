from .lexer import *


@final
class AstPrinter(Visitor[str]):
	def visit_binary(self, expr: Binary) -> str:
		return f"{expr.operator.lexeme} ({expr.left.run_against(self)}) ({expr.right.run_against(self)})"
	
	def visit_grouping(self, expr: Grouping) -> str:
		return f"(group {expr.expression.run_against(self)})"

	def visit_literal(self, expr: Literal) -> str:
		return f"{expr.value}"
	
	def visit_unary(self, expr: Unary) -> str:
		return f"{expr.operator.lexeme} {expr.right.run_against(self)}"


if __name__ == "__main__":
	printer = AstPrinter()
	expr = Binary(
		Unary(Token(TokenType.MINUS, '-', 0, None), Literal(123)),
		Grouping(Literal(54.234)),
		Token(TokenType.STAR, '*', 0, None)
	)
	print(expr.run_against(printer))
	print()