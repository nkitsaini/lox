from .prelude import *
import argparse
from .scanner.scanner import Scanner
from .scanner.parser import Parser
# from .scanner.ast_printer import AstPrinter
from .scanner.ast_interpreter import AstInterpreter
from . import lox


@dataclass
class Runner:
	interpreter: AstInterpreter = field(default_factory=AstInterpreter)
	def run(self, source: str):
		scanner = Scanner(source)
		tokens = scanner.scan_tokens();
		# print(tokens)
		statements = Parser(tokens).parse()
		if lox.had_error:
			# print("err")
			return
		assert statements is not None
		
		self.interpreter.interpret(statements)

	def run_file(self, file_path: str):
		self.run(Path(file_path).read_text())
		if lox.had_error:
			exit(65)
		if lox.had_runtime_error:
			exit(70)
	
	def run_prompt(self):
		while True:
			try:
				line = input("> ")
				self.run(line)

				lox.had_error = False
			except EOFError:
				break

	def start(self):
		parser = argparse.ArgumentParser()
		parser.add_argument('source', nargs="?", default=None)
		args = parser.parse_args()
		if args.source:
			self.run_file(args.source)
		else:
			self.run_prompt()
