from enum import Enum, auto
from ..prelude import *


@dataclass
class Token:
	token_type: 'TokenType'
	lexeme: str
	line: int

	# Python representation of lox types
	literal_val: Any = None

	def __hash__(self) -> int:
		return id(self)

class TokenType(Enum):

	# Single Character Tokens
	LEFT_PARAN = auto() # (
	RIGHT_PARAN = auto() # )
	LEFT_BRACE = auto() # {
	RIGHT_BRACE = auto() # }
	COMMA = auto()
	DOT = auto()
	MINUS = auto()
	PLUS = auto()
	SEMICOLON = auto()
	SLASH = auto()
	STAR = auto()


	# One or Two character tokens
	BANG = auto()
	BANG_EQUAL = auto()
	EQUAL = auto()
	EQUAL_EQUAL = auto()
	GREATER = auto()
	GREATER_EQUAL = auto()
	LESS = auto()
	LESS_EQUAL = auto()

	# Literals
	IDENTIFIER = auto()
	STRING = auto()
	NUMBER = auto()

	# Keywords
	AND = auto()
	CLASS = auto()
	ELSE = auto()
	FALSE = auto()
	FUN = auto()
	FOR = auto()
	IF = auto()
	NIL = auto()
	OR = auto()
	PRINT = auto()
	RETURN = auto()
	SUPER = auto()
	THIS = auto()
	TRUE = auto()
	VAR = auto()
	WHILE = auto()
	BREAK = auto()

	# Implicity
	EOF = auto()
	

