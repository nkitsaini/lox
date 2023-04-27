from pylox.prelude import *
from .tokens import Token, TokenType
from .lexer import *
from . import lexer
from .. import lox

class ParseError(RuntimeError):
	pass

@dataclass
class Parser:
	tokens: List[Token]
	current: int = 0

	def expression(self) -> BaseExpr:
		return self.equality()
	
	def peek_opt(self):
		assert self.current < len(self.tokens)
		if self.current < len(self.tokens):
			return self.tokens[self.current]
		return None
	def peek(self):
		assert self.current < len(self.tokens)
		return self.tokens[self.current]
	
	def match(self, *args: TokenType):
		return (a:=self.peek_opt()) != None and a.token_type in args

	def take(self) -> Token:
		self.current += 1
		return self.tokens[self.current - 1]

	def consume(self, ty: TokenType, error: str):
		token = self.take()
		if token.token_type != ty:
			raise self.error(token, error)
		return token
		
	
	def parse(self):
		statements: List[Statement] = []
		try:
			while (tok:=self.peek_opt()) and tok.token_type != TokenType.EOF:
				# print(self.peek_opt())
				statements.append(self.declaration())
			return statements
		except ParseError:
			return None
	
	def statement(self) -> Statement:
		if (token:=self.peek_opt()) != None and token.token_type == TokenType.PRINT:
			return self.print_statement()
		else:
			return self.expression_statement()
	
	def declaration(self) -> Statement:
		# Either var delaration or statement
		if (token:=self.peek_opt()) != None and token.token_type == TokenType.VAR:
			return self.var_statement()
		else:
			return self.statement()

	def var_statement(self):
		self.consume(TokenType.VAR, "Interpreter bug!")
		identifier = self.consume(TokenType.IDENTIFIER, "Expected identifier in var block")
		identifier = self.take()
		if self.peek().token_type == TokenType.EQUAL:
			self.take()
			rv = Var(self.expression(), identifier)
			self.consume(TokenType.SEMICOLON, "Semicolon expected in variable declaration")
			return rv
		elif self.peek().token_type == TokenType.SEMICOLON:
			self.take()
			return Var(None, identifier)
		else:
			raise self.error(self.take(), "Unexpected token in var declaration")
	def print_statement(self):
		self.consume(TokenType.PRINT, "This is interpreter bug!")
		rv = Print(self.expression())
		self.consume(TokenType.SEMICOLON, "Expected ; after print statement")
		return rv

	def expression_statement(self):
		rv = Expression(self.expression())
		self.consume(TokenType.SEMICOLON, "Expected ; after print statement")
		return rv
	
	def error(self, token: Token, msg: str):
		lox.errorToken(token, msg)
		return ParseError()
	
	def synchronize(self):
		"""
		Consume tokens until End of line, end of file or start of a new statement
		"""
		self.take()
		while (token:=self.peek_opt()):

			if token.token_type == TokenType.SEMICOLON:
				self.take()
				return

			if token.token_type in [TokenType.CLASS, TokenType.FOR, TokenType.IF, TokenType.VAR, TokenType.WHILE, TokenType.PRINT, TokenType.RETURN, TokenType.FUN]:
				return
		

	
	def equality(self) -> BaseExpr:
		expr = self.comparison()

		while (self.match(TokenType.EQUAL_EQUAL, TokenType.BANG_EQUAL)):
			operator = self.take()
			right = self.comparison()
			expr = Binary(expr, right, operator)
		return expr
	
	def comparison(self) -> BaseExpr:
		expr = self.term()

		while (self.match(TokenType.GREATER, TokenType.GREATER_EQUAL, TokenType.LESS, TokenType.LESS_EQUAL)):
			operator = self.take()
			right = self.term()
			expr = Binary(expr, right, operator)
		return expr

	def term(self) -> BaseExpr:
		expr = self.factor()

		while (self.match(TokenType.MINUS, TokenType.PLUS)):
			operator = self.take()
			right = self.factor()
			expr = Binary(expr, right, operator)
		return expr

	def factor(self) -> BaseExpr:
		expr = self.unary()
		while (self.match(TokenType.SLASH, TokenType.STAR)):
			operator = self.take()
			right = self.unary()
			expr = Binary(expr, right, operator)
		return expr

	def unary(self) -> BaseExpr:
		if (self.match(TokenType.MINUS, TokenType.BANG)):
			self.take()
			return Unary(self.take(), self.unary())
		else:
			return self.primary()

	def primary(self) -> BaseExpr:
		if self.match(TokenType.LEFT_PARAN):
			self.take()
			rv = Grouping(self.expression())
			self.consume(TokenType.RIGHT_PARAN, "Expected ')' After expression") # consuming ending brace
			return rv
		elif self.match(TokenType.NUMBER, TokenType.STRING, TokenType.NIL, TokenType.TRUE, TokenType.FALSE):
			return lexer.Literal(self.take().literal_val)
		elif self.match(TokenType.IDENTIFIER):
			return lexer.Variable(self.take())
		token = self.take()
		raise self.error(token, f"Expected to find literal or group but found {token.lexeme}")