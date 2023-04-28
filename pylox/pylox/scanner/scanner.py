from .tokens import TokenType, Token
from .. import lox
from ..prelude import *


@dataclass
class Scanner:
	source: str
	position: int = 0
	line: int = 0

	def scan_tokens(self) -> List[Token]:
		tokens: List[Token] = []

		self.consume_empty()
		while self.peek_opt() != None:
			token = self.scan_next()
			if token != None:
				tokens.append(token)
			self.consume_empty()

		tokens.append(
			Token(TokenType.EOF, '', self.line),
		)

		return tokens

	def consume_empty(self):
		while self.position < len(self.source) and self.source[self.position] in [' ', '\t', '\n']:
			if self.source[self.position] == '\n':
				self.line += 1
			self.position += 1

	def peek_opt(self) -> Optional[str]:
		# while self.position < len(self.source) and self.source[self.position] in [' ', '\t', '\n']:
		# 	if self.source[self.position] == '\n':
		# 		self.line += 1
		# 	self.position += 1
		if self.position < len(self.source):
			return self.source[self.position]
	

	def take(self, count: int = 1) -> str:
		rv = self.source[self.position:self.position+count]
		self.position += count
		return rv

	def take_line(self, count: int = 1) -> str:
		index = self.source.find('\n', self.position+1)
		if index == -1:
			index = len(self.source)
		else:
			self.line += 1
			index += 1  # consume the ending /n
		
		rv = self.source[self.position:index]
		self.position = index
		return rv

	def peek_next(self) -> Optional[str]:
		if self.position +1 < len(self.source):
			return self.source[self.position+1]
	
	def get_line_no(self) -> int:
		return 0
	
	def consume_digits(self) -> str:
		rv = ""
		while (a:=self.peek_opt()) != None and a.isnumeric():
			rv += self.take()
		return rv

	def consume_string(self) -> str:
		rv = self.take() # First "
		while (a:=self.peek_opt()) != None and a != '"' :
			if a == "\n":
				self.line += 1
			rv += self.take()
		if self.peek_opt() != '"':
			lox.error(self.line, "Unterminated String")
			return rv
		rv += self.take() # End "
		return rv
	
	def scan_next(self) -> Optional[Token]: # consumes comment if token is comment
		one_char_map = {
			'(': TokenType.LEFT_PARAN,
			')': TokenType.RIGHT_PARAN,
			'{': TokenType.LEFT_BRACE,
			'}': TokenType.RIGHT_BRACE,
			',': TokenType.COMMA,
			'.': TokenType.DOT,
			'-': TokenType.MINUS,
			'+': TokenType.PLUS,
			';': TokenType.SEMICOLON,
			# '/': TokenType.SLASH, # Need to do something related to commenting in lox
			'*': TokenType.STAR
		}


		line_no = self.line

		# Single character
		if self.peek_opt() in one_char_map:
			char = self.take()
			return Token(
				one_char_map[char],
				char,
				line_no
			)

		match self.peek_opt():
			# Either one or two characters (peek next)
			case '!':
				if self.peek_next() == '=':
					return Token(TokenType.BANG_EQUAL, self.take(2), line_no)
				else:
					return Token(TokenType.BANG, self.take(1), line_no)
			
			case '=':
				if self.peek_next() == '=':
					return Token(TokenType.EQUAL_EQUAL, self.take(2), line_no)
				else:
					return Token(TokenType.EQUAL, self.take(1), line_no)
			
			case '>':
				if self.peek_next() == '=':
					return Token(TokenType.GREATER_EQUAL, self.take(2), line_no)
				else:
					return Token(TokenType.GREATER, self.take(1), line_no)

			case '<':
				if self.peek_next() == '=':
					return Token(TokenType.LESS_EQUAL, self.take(2), line_no)
				else:
					return Token(TokenType.LESS, self.take(1), line_no)
			case '/': # 
				if self.peek_next() == '/':
					self.take_line()
					return None
				elif self.peek_next() == '*': # multiline
					self.take()
					self.take()

					while self.peek_next() != None and (self.peek_opt() != '*' or self.peek_next() != '/'):
						self.take()

					if self.peek_next() == None:
						lox.error(line_no, "Unterminated multiline comment")
						return
					self.take() # take *
					self.take() # take /
					return None
				else:
					return Token(TokenType.SLASH, self.take(1), line_no)
				
			case _ as char if char != None and char.isnumeric():
				rv = self.consume_digits()
				if self.peek_opt() == "." and (n:=self.peek_next()) != None and n.isnumeric():
					rv += self.take()
					rv += self.consume_digits()
				return Token(TokenType.NUMBER, rv, line_no, eval(rv))

			case '"':
				rv = self.consume_string()
				return Token(TokenType.STRING, rv, line_no, rv[1:-1])

			case _ as char if char != None and (char.isalpha() or  char == "_"):
				lexeme = self.take()
				while (a:=self.peek_opt()) != None and (a.isalnum() or a == "_"):
					char = self.take()
					lexeme += char

				reserved_identifier = {
					"and": TokenType.AND,
					"class": TokenType.CLASS,
					"else": TokenType.ELSE,
					"false": TokenType.FALSE,
					"true": TokenType.TRUE,
					"fun": TokenType.FUN,
					"for": TokenType.FOR,
					"if": TokenType.IF,
					"nil": TokenType.NIL,
					"or": TokenType.OR,
					"print": TokenType.PRINT,
					"return": TokenType.RETURN,
					"super": TokenType.SUPER,
					"this": TokenType.THIS,
					"var": TokenType.VAR,
					"while": TokenType.WHILE,
					"break": TokenType.BREAK,
				}

				if lexeme in reserved_identifier:
					literal_val = None
					if lexeme == "true":
						literal_val = True
					else:
						literal_val = False
					return Token(reserved_identifier[lexeme], lexeme, line_no, literal_val)
				else:
					return Token(TokenType.IDENTIFIER, lexeme, line_no)

			case _:
				
				lox.error(line_no, "Unexpected token: " + self.take())