#include <stdio.h>
#include <string.h>

#include "scanner.h"
#include "common.h"

typedef struct {
	const char* start;
	const char* current;
	int line;
} Scanner;

Scanner scanner;


void initScanner(const char *source)
{
	scanner.start = source;
	scanner.current = source;
	scanner.line = 1;
}

static bool isAtEnd() {
	return *scanner.current == '\0';
}

static Token errorToken(const char* message) {
	Token token;
	token.type = TOKEN_ERROR;
	token.line = scanner.line;
	token.start = message;
	token.length = (int)(strlen(message));
	return token;
}

static Token makeToken(TokenType type) {
	Token token;
	token.type = type;
	token.line = scanner.line;
	token.start = scanner.start;
	token.length = (int)(scanner.current - scanner.start);
	return token;
}

static char advance() {
	scanner.current += 1;
	return scanner.current[-1];
}

static bool match(char c) {
	if (isAtEnd()) {
		return false;
	}
	if (*scanner.current == c) {
		advance();
		return true;
	}
	return false;
}
static char peek() {
	return *scanner.current;
}
static char peekNext() {
	if (isAtEnd()) return '\0';
	return scanner.current[1];
}

static void skipWhitespace() {
	for (;;) {
		char c= peek();
		switch (c) {
			case ' ':
			case '\r':
			case '\t':
				advance();
				break;
			case '\n':
				scanner.line++;
				advance();
				break;
			case '/':
				if (peekNext() == '/') {
					while (peek() != '\n' && !isAtEnd()) advance();
				}
				break;
			default:
				return;
		}
	}
}

static Token string() {
	while (peek() != '"' && !isAtEnd()) {
		if (peek()  == '\n') scanner.line ++;
		advance();
	}
	if (isAtEnd()) return errorToken("Unterminated String");

	advance();
	return makeToken(TOKEN_STRING);
}

static Token number() {
	while (isDigit(peek())) advance();

	// Fractional part
	if (peek() == '.' && isDigit(peekNext()))  {
		advance();
		while (isDigit(peek())) advance();
	}
	return makeToken(TOKEN_NUMBER);
}

static bool isDigit(char c) {
	return c >= '0' && c <= '9';
}
static bool isAlpha(char c) {
	return (c >= 'a' && c <= 'z') ||
			(c >= 'A' && c <= 'Z') ||
			c == '_';
}

static TokenType identifierType() {

}
static Token identifier() {
	while (isAlpha(peek()) || isDigit(peek())) advance();
	return makeToken(idenitiferType());
}

Token scanToken()
{
	skipWhitespace();
	scanner.start = scanner.current;
	if (isAtEnd()) return makeToken(TOKEN_EOF);
	char c = advance();
	if (isDigit(c)) {
		return number();
	}
	if (isAlpha(c)) {
		return identifier();
	}
	switch (c) {

		// Single character
		case '(': return makeToken(TOKEN_LEFT_PAREN);
		case ')': return makeToken(TOKEN_RIGHT_PAREN);
		case '[': return makeToken(TOKEN_LEFT_BRACE);
		case ']': return makeToken(TOKEN_RIGHT_BRACE);
		case ',': return makeToken(TOKEN_COMMA);
		case '.': return makeToken(TOKEN_DOT);
		case '-': return makeToken(TOKEN_MINUS);
		case '+': return makeToken(TOKEN_PLUS);
		case ';': return makeToken(TOKEN_SEMICOLON);
		case '/': return makeToken(TOKEN_SLASH);
		case '*': return makeToken(TOKEN_STAR);

		// One or two character
		case '!': return makeToken(match('=')?TOKEN_BANG_EQUAL:TOKEN_BANG);
		case '=': return makeToken(match('=')?TOKEN_EQUAL_EQUAL:TOKEN_EQUAL);
		case '>': return makeToken(match('=')?TOKEN_GREATER_EQUAL:TOKEN_GREATER);
		case '<': return makeToken(match('=')?TOKEN_LESS_EQUAL:TOKEN_LESS);

		// Literals
		case '"': return string();

	}
	return errorToken("Unexpected character.");
}
