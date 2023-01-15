from .prelude import *
import argparse
from .scanner.scanner import Scanner


@dataclass
class Runner:
	had_error: bool = False

	def error(self, line: int, msg: str):
		self.had_error = True
		print(f"Line {line} | {msg}")

	def run(self, source: str):
		scanner = Scanner(source)
		tokens = scanner.scan_tokens();
		for token in tokens:
			print(token)

	def run_file(self, file_path: str):
		self.run(Path(file_path).read_text())
		if self.had_error:
			exit(65)
	
	def run_prompt(self):
		while True:
			try:
				line = input("> ")
				self.run(line)

				self.had_error = False
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
