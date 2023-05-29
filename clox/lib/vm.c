#include "vm.h"
#include "common.h"
#include "compiler.h"
#include "debug.h"
#include "memory.h"
#include "string.h"
#include <stdarg.h>
VM vm;

static void resetStack()
{
	vm.stackTop = vm.stack;
}

static void runtimeError(const char *format, ...)
{
	// TODO: I don't understand this. I wrote it just by looking at book
	va_list args;
	va_start(args, format);
	vfprintf(stderr, format, args);
	va_end(args);
	fputs("\n", stderr);
	size_t instruction = vm.ip - vm.chunk->code - 1;
	int line = vm.chunk->lines[instruction];
	fprintf(stderr, "[line %d] in script\n", line);
	resetStack();
}

static Value peek(int distance)
{
	return vm.stackTop[-1 - distance];
}

static bool isFalsey(Value value)
{
	return IS_NIL(value) || (IS_BOOL(value) && !AS_BOOL(value));
}
static void concatenate()
{
	ObjString *b = AS_STRING(pop());
	ObjString *a = AS_STRING(pop());
	int length = a->length + b->length;
	char *chars = ALLOCATE(char, length + 1);
	memcpy(chars, a->chars, a->length);
	memcpy(chars + a->length, b->chars, b->length);
	chars[length] = '\0';
	ObjString *result = takeString(chars, length);
	push(OBJ_VAL(result));
}

void initVM()
{
	vm.stack = NULL;
	vm.stack_length = 0;
	vm.objects = NULL;
	resetStack();
}

void freeVM()
{
	// INVESTIGATE: Chapter 19 does not mention other code, how did I get it?
	FREE_ARRAY(Value, vm.stack, vm.stack_length);
	vm.stack_length = 0;
	vm.stack = NULL;
	vm.stackTop = NULL;
	freeObjects();
}

void push(Value value)
{
	if (vm.stack_length == 0 || vm.stackTop == (vm.stack + vm.stack_length))
	{
		int new_size = GROW_CAPACITY(vm.stack_length);
		vm.stack = GROW_ARRAY(Value, vm.stack, vm.stack_length, new_size);
		vm.stackTop = vm.stack + vm.stack_length;
		vm.stack_length = new_size;
	}
	*vm.stackTop = value;
	vm.stackTop++;
}

Value pop()
{
	vm.stackTop--;
	return *vm.stackTop;
}

static InterpretResult run()
{
#define READ_BYTE() (*vm.ip++)
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
// we pop `b` first because it'll be inserted last.
// In expression `3 + 1`, we'll first insert `3` and then `1` to stack
// So we need to assign `1` (top most element) to b and `3` to a
#define BINARY_OP(valueType, op)                        \
	do                                                  \
	{                                                   \
		if (!IS_NUMBER(peek(0)) || !IS_NUMBER(peek(1))) \
		{                                               \
			runtimeError("Operands must be numbers;");  \
			return INTERPRET_RUNTIME_ERROR;             \
		}                                               \
		double b = AS_NUMBER(pop());                    \
		double a = AS_NUMBER(pop());                    \
		push(valueType(a op b));                        \
	} while (false)

	for (;;)
	{

#ifdef DEBUG_TRACE_EXECUTION
		printf("        ");
		for (Value *slot = vm.stack; slot < vm.stackTop; slot++)
		{
			printf("[ ");
			printValue(*slot);
			printf(" ]");
		}
		printf("\n");
		disassembleInstruction(vm.chunk, (int)(vm.ip - vm.chunk->code));
#endif
		uint8_t instruction;
		switch (instruction = READ_BYTE())
		{
		case OP_CONSTANT:
		{
			Value constant = READ_CONSTANT();
			printValue(constant);
			push(constant);
			printf("\n");
			break;
		}
		case OP_NEGATE:
		{
			// Little bit faster then push(-pop());
			// Used hyperfine (500_000 negations create chunk + disassemble + execute chunk)
			//
			// Using inplace modification:
			//     Time (mean ± σ):     534.5 ms ±   5.4 ms    [User: 508.4 ms, System: 36.3 ms]
			//     Range (min … max):   525.0 ms … 543.8 ms    10 runs
			// Using Pop:
			//     Time (mean ± σ):     549.7 ms ±   7.9 ms    [User: 526.0 ms, System: 34.5 ms]
			//     Range (min … max):   542.2 ms … 567.1 ms    10 runs

			if (!IS_NUMBER(peek(0)))
			{
				runtimeError("Operand must be a number");
				return INTERPRET_RUNTIME_ERROR;
			}
			push(NUMBER_VAL(-AS_NUMBER(pop())));
			break;
		}
		case OP_NOT:
		{
			push(BOOL_VAL(isFalsey(pop())));
			break;
		}
		case OP_NIL:
			push(NIL_VAL);
			break;
		case OP_TRUE:
			push(BOOL_VAL(true));
			break;
		case OP_FALSE:
			push(BOOL_VAL(false));
			break;
		case OP_EQUAL:
		{
			Value b = pop();
			Value a = pop();
			push(BOOL_VAL(valuesEqual(a, b)));
			break;
		}
		case OP_GREATER:
			BINARY_OP(BOOL_VAL, >);
			break;
		case OP_LESS:
			BINARY_OP(BOOL_VAL, <);
			break;
		case OP_ADD:
		{
			if (IS_STRING(peek(0)) && IS_STRING(peek(1)))
			{
				concatenate();
			}
			else if (IS_NUMBER(peek(0)) && IS_NUMBER(peek(1)))
			{
				double b = AS_NUMBER(pop());
				double a = AS_NUMBER(pop());
				push(NUMBER_VAL(a + b));
			}
			else
			{
				runtimeError("Operands must be two numbers or two strings.");
				return INTERPRET_RUNTIME_ERROR;
			}
			break;
		}
		case OP_SUBTRACT:
			BINARY_OP(NUMBER_VAL, -);
			break;
		case OP_MULTIPLY:
			BINARY_OP(NUMBER_VAL, *);
			break;
		case OP_DIVIDE:
			BINARY_OP(NUMBER_VAL, /);
			break;
		case OP_RETURN:
		{
			printValue(pop());
			return INTERPRET_OK;
		}
		}
	}

#undef READ_BYTE
#undef READ_CONSTANT
#undef BINARY_OP
}

InterpretResult interpret(char *source)
{
	Chunk chunk;
	initChunk(&chunk);

	if (!compile(source, &chunk))
	{
		freeChunk(&chunk);
		return INTERPRET_COMPILE_ERROR;
	}
	vm.chunk = &chunk;
	vm.ip = vm.chunk->code;
	InterpretResult result = run();
	freeChunk(&chunk);
	return result;
}
