from .tokens import TokenType, Token
from ..prelude import *


@dataclass
class Scanner:
	source: str

	def scan_tokens(self) -> List[Token]:
		tokens = []



		tokens.append(
			Token(TokenType.EOF, '', None, 2),
		)

		return tokens