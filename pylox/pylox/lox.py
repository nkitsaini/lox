from typing import TYPE_CHECKING
import colorama

if TYPE_CHECKING:
	from .scanner.tokens import Token
	from .scanner.ast_interpreter import LoxRuntimeError

had_error = False
had_runtime_error = False

def error(line: int, msg: str):
	global had_error
	had_error = True
	
	print(colorama.Fore.RED + "[ERROR] " + colorama.Fore.RESET + f"{colorama.Fore.GREEN}Line {line+1} |{colorama.Fore.RESET} {msg}")

def runtime_error(error: 'LoxRuntimeError'):
	global had_runtime_error
	had_runtime_error = True
	
	print(colorama.Fore.RED + "[RUNTIME ERROR] " + colorama.Fore.RESET + f"{colorama.Fore.GREEN}Line {error.token.line+1} |{colorama.Fore.RESET} {error.args[0]}")

def errorToken(token: 'Token', msg: str):
	error(token.line, msg)
	