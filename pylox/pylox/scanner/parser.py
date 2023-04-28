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
		return self.assignment()

	def assignment(self) -> BaseExpr:
		# expr = self.equality()
		expr = self.logic_or()
		if self.match(TokenType.EQUAL):
			equals = self.take()
			value = self.assignment()
			# More checks other then `Variable` will be added here
			# To accomodate cases like: `my_set[1+2].value = 3`
			if isinstance(expr, Variable):
				r= Assignment(expr.name, value)
				return r
			self.error(equals, "Invalid Assignment target")
		return expr
	
	def logic_or(self) -> BaseExpr:
		expr = self.logic_and()
		while (self.match(TokenType.OR)):
			operator = self.take()
			right = self.logic_and()
			expr = Logical(expr, operator, right)
		return expr


	def logic_and(self) -> BaseExpr:
		expr = self.equality()
		while (self.match(TokenType.OR)):
			operator = self.take()
			right = self.equality()
			expr = Logical(expr, operator, right)
		return expr
	
	def match_next(self, *args: TokenType):
		return (a:=self.peek_next_opt()) != None and a.token_type in args

	def peek_opt(self):
		assert self.current < len(self.tokens)
		if self.current < len(self.tokens):
			return self.tokens[self.current]
		return None

	def peek_next_opt(self):
		if self.current < len(self.tokens) -1:
			return self.tokens[self.current+1]
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
				statements.append(self.declaration())
			return statements
		except ParseError:
			return None
	
	def statement(self) -> Statement:
		if self.match(TokenType.PRINT):
			return self.print_statement()
		if self.match(TokenType.LEFT_BRACE):
			return self.block_statement()
		if self.match(TokenType.IF):
			return self.if_statement()
		if self.match(TokenType.WHILE):
			return self.while_statement()
		if self.match(TokenType.FOR):
			return self.for_statement()
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
		# identifier = self.take()
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

	def block_statement(self):
		self.take()
		statements: List[Statement] = []

		# TODO: how will nested work?
		# I guess it'll work out-of-the-box due to delcaration call
		while not self.match(TokenType.RIGHT_BRACE) and self.peek_opt() is not None:
			statements.append(self.declaration())
		
		self.consume(TokenType.RIGHT_BRACE, "Expected closing brace for block statement")
		return Block(statements)


	def if_statement(self):
		self.take() # if
		self.consume(TokenType.LEFT_PARAN, "`if` should be followed by `(`")
		condition = self.expression()
		self.consume(TokenType.RIGHT_PARAN, "Unclosed `if` parans `)`")
		inner = self.statement()

		else_inner = None
		if (self.match(TokenType.ELSE)):
			self.take()
			else_inner = self.statement()
		return If(condition, inner, else_inner)

	def while_statement(self):
		self.take() # while
		self.consume(TokenType.LEFT_PARAN, "`while` should be followed by `(`")
		condition = self.expression()
		self.consume(TokenType.RIGHT_PARAN, "Unclosed `while` parans `)`")
		inner = self.statement()

		return While(condition, inner)

	def for_statement(self):
		self.take() # for
		self.consume(TokenType.LEFT_PARAN, "`for` should be followed by `(`")
		initializer: Optional[Statement] = None
		if (self.match(TokenType.SEMICOLON)):
			self.take()
		elif (self.match(TokenType.VAR)):
			initializer = self.var_statement()
		else:
			initializer = self.expression_statement()
		

		condition: Optional[BaseExpr] = None
		if (not self.match(TokenType.SEMICOLON)):
			condition = self.expression()
		
		self.consume(TokenType.SEMICOLON, "Expected semicolon after condition")
		increment: Optional[BaseExpr] = None
		if (not self.match(TokenType.RIGHT_PARAN)):
			increment = self.expression()
		
		self.consume(TokenType.RIGHT_PARAN, "Expected ) after step in for loop")

		body = self.statement()
		rv = body
		if increment:
			rv = Block([rv, Expression(increment)])
		if condition:
			rv = While(condition, rv)
		if initializer:
			rv = Block([initializer, rv])
		return rv


		
		
		# condition = self.expression()
		# self.consume(TokenType.RIGHT_PARAN, "Unclosed `for` parans `)`")
		# inner = self.statement()

		# return While(condition, inner)

	def expression_statement(self):
		rv = Expression(self.expression())
		self.consume(TokenType.SEMICOLON, "Expected ; after Expression")
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
		if self.match(TokenType.RIGHT_PARAN):
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