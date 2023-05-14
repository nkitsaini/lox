#include <stdio.h>
#include "common.h"
#include <stdlib.h>
#include "scanner.h"
#include "compiler.h"

#ifdef DEBUG_PRINT_CODE
#include "debug.h"
#endif

typedef struct
{
	Token current;
	Token previous;
	bool hadError;
	bool panicMode; // Did I forget to implement this in pylox?
} Parser;

typedef enum
{
	PREC_NONE,
	PREC_ASSIGNMENT,
	PREC_OR,
	PREC_AND,
	PREC_EQUALITY,
	PREC_COMPARISON,
	PREC_TERM,
	PREC_FACTOR,
	PREC_UNARY,
	PREC_CALL,
	PREC_PRIMARY,
} Precedence;

typedef void (*ParseFn)();

typedef struct
{
	ParseFn prefix;
	ParseFn inifix;
	Precedence precedence;
} ParseRule;

Parser parser;
Chunk *compilingChunk;

static Chunk *currentChunk()
{
	return compilingChunk;
}

static void errorAt(Token *token, const char *message)
{
	if (parser.panicMode)
		return;
	parser.panicMode = true;
	fprintf(stderr, "[line %d] Error", token->line);

	if (token->type == TOKEN_EOF)
	{
		fprintf(stderr, " at end");
	}
	else if (token->type == TOKEN_ERROR)
	{
		// Nothing, will always print the message
	}
	else
	{
		fprintf(stderr, " at '%.*s'", token->length, token->start);
	}
	fprintf(stderr, ": %s\n", message);
	parser.hadError = true;
}

static void errorAtCurrent(const char *message)
{
	errorAt(&parser.current, message);
}

static void error(const char *message)
{
	errorAt(&parser.previous, message);
}

static void advance()
{
	parser.previous = parser.current;

	for (;;)
	{
		parser.current = scanToken();
		if (parser.current.type != TOKEN_ERROR)
			break;

		errorAtCurrent(parser.current.start);
	}
}

static void consume(TokenType type, const char *message)
{
	if (parser.current.type == type)
	{
		advance();
		return;
	}
	errorAtCurrent(message);
}

static void emitByte(uint8_t byte)
{
	writeChunk(currentChunk(), byte, parser.previous.line);
}

static void emitBytes(uint8_t byte, uint8_t byte2)
{
	emitByte(byte);
	emitByte(byte2);
}

static void emitReturn()
{
	emitByte(OP_RETURN);
}

static void endCompiler()
{
	emitReturn();
#ifdef DEBUG_PRINT_CODE
	if (!parser.hadError)
	{
		disassembleChunk(currentChunk(), "code");
	}
#endif
}

static void expression();
static ParseRule *getRule(TokenType type);
static void parsePrecedence(Precedence precedence);

static void binary()
{
	TokenType operatorType = parser.previous.type;
	ParseRule *rule = getRule(operatorType);
	parsePrecedence((Precedence)(rule->precedence + 1));

	switch (operatorType)
	{
	case TOKEN_PLUS:
		emitByte(OP_ADD);
		break;
	case TOKEN_MINUS:
		emitByte(OP_SUBTRACT);
		break;
	case TOKEN_STAR:
		emitByte(OP_MULTIPLY);
		break;
	case TOKEN_SLASH:
		emitByte(OP_DIVIDE);
		break;
	default:
		return; // unreachable
	}
}

static uint8_t makeConstant(Value value)
{
	int constant = addConstant(currentChunk(), value);
	if (constant > UINT8_MAX)
	{
		error("Too many constants in one chunk");
		return 0;
	}
	return (uint8_t)constant;
}

static void emitConstant(Value value)
{
	emitBytes(OP_CONSTANT, makeConstant(value));
}

static void number()
{
	double value = strtod(parser.previous.start, NULL); // should it be `start + length` instead of NULL?
	emitConstant(value);
}

static void unary()
{
	TokenType operatorType = parser.previous.type;
	parsePrecedence(PREC_UNARY);
	switch (operatorType)
	{
	case TOKEN_MINUS:
		emitByte(OP_NEGATE);
		break;
	default:
		return;
	}
}

static void grouping()
{
	expression();
	consume(TOKEN_RIGHT_PAREN, "Expect `)` after expression.");
}

ParseRule rules[] = {
	[TOKEN_LEFT_PAREN] = {grouping, NULL, PREC_NONE},
	[TOKEN_RIGHT_PAREN] = {NULL, NULL, PREC_NONE},
	[TOKEN_LEFT_BRACE] = {NULL, NULL, PREC_NONE},
	[TOKEN_RIGHT_BRACE] = {NULL, NULL, PREC_NONE},
	[TOKEN_COMMA] = {NULL, NULL, PREC_NONE},
	[TOKEN_DOT] = {NULL, NULL, PREC_NONE},
	[TOKEN_MINUS] = {unary, binary, PREC_TERM},
	[TOKEN_PLUS] = {NULL, binary, PREC_TERM},
	[TOKEN_SEMICOLON] = {NULL, NULL, PREC_TERM},
	[TOKEN_SLASH] = {NULL, binary, PREC_FACTOR},
	[TOKEN_STAR] = {NULL, binary, PREC_FACTOR},
	[TOKEN_BANG] = {NULL, NULL, PREC_NONE},
	[TOKEN_BANG_EQUAL] = {NULL, NULL, PREC_NONE},
	[TOKEN_EQUAL] = {NULL, NULL, PREC_NONE},
	[TOKEN_EQUAL_EQUAL] = {NULL, NULL, PREC_NONE},
	[TOKEN_GREATER] = {NULL, NULL, PREC_NONE},
	[TOKEN_GREATER_EQUAL] = {NULL, NULL, PREC_NONE},
	[TOKEN_LESS] = {NULL, NULL, PREC_NONE},
	[TOKEN_LESS_EQUAL] = {NULL, NULL, PREC_NONE},
	[TOKEN_IDENTIFIER] = {NULL, NULL, PREC_NONE},
	[TOKEN_STRING] = {NULL, NULL, PREC_NONE},
	[TOKEN_NUMBER] = {number, NULL, PREC_NONE},
	[TOKEN_AND] = {NULL, NULL, PREC_NONE},
	[TOKEN_CLASS] = {NULL, NULL, PREC_NONE},
	[TOKEN_ELSE] = {NULL, NULL, PREC_NONE},
	[TOKEN_FALSE] = {NULL, NULL, PREC_NONE},
	[TOKEN_FOR] = {NULL, NULL, PREC_NONE},
	[TOKEN_FUN] = {NULL, NULL, PREC_NONE},
	[TOKEN_IF] = {NULL, NULL, PREC_NONE},
	[TOKEN_NIL] = {NULL, NULL, PREC_NONE},
	[TOKEN_OR] = {NULL, NULL, PREC_NONE},
	[TOKEN_PRINT] = {NULL, NULL, PREC_NONE},
	[TOKEN_RETURN] = {NULL, NULL, PREC_NONE},
	[TOKEN_SUPER] = {NULL, NULL, PREC_NONE},
	[TOKEN_THIS] = {NULL, NULL, PREC_NONE},
	[TOKEN_TRUE] = {NULL, NULL, PREC_NONE},
	[TOKEN_VAR] = {NULL, NULL, PREC_NONE},
	[TOKEN_WHILE] = {NULL, NULL, PREC_NONE},
	[TOKEN_ERROR] = {NULL, NULL, PREC_NONE},
	[TOKEN_EOF] = {NULL, NULL, PREC_NONE},
};

static void
parsePrecedence(Precedence precendence)
{
	advance();
	ParseFn prefixRule = getRule(parser.previous.type)->prefix;
	if (prefixRule == NULL)
	{
		error("Expect Expression");
		return;
	}
	// TODO(Chapter17-3): show how the mixfix operators like ( conditional ?: ) can be operated here
	prefixRule();

	while (precendence <= getRule(parser.current.type)->precedence)
	{
		advance();
		ParseFn infinixRule = getRule(parser.previous.type)->inifix;
		infinixRule();
	}
}
static ParseRule *getRule(TokenType type)
{
	return &rules[type];
}

static void expression()
{
	parsePrecedence(PREC_ASSIGNMENT);
}

bool compile(const char *source, Chunk *chunk)
{
	initScanner(source);
	compilingChunk = chunk;

	parser.hadError = false;
	parser.panicMode = false;
	advance();
	expression();

	consume(TOKEN_EOF, "Expect end of expression");
	endCompiler();
	return !parser.hadError;
}