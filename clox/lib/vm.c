#include "common.h"
#include "vm.h"
#include "debug.h"
#include "memory.h"
VM vm;

static void resetStack() {
	vm.stackTop = vm.stack;
}

void initVM() {
	vm.stack = NULL;
	vm.stack_length = 0;
	resetStack();
}
void freeVM() {

}

void push(Value value)
{
	if (vm.stack_length == 0 || vm.stackTop == (vm.stack + vm.stack_length)) {
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

static InterpretResult run() {
	#define READ_BYTE() (*vm.ip++)
	#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])

	// we pop `b` first because it'll be inserted last.
	// In expression `3 + 1`, we'll first insert `3` and then `1` to stack
	// So we need to assign `1` (top most element) to b and `3` to a
	#define BINARY_OP(op) \
		do { \
			double b = pop(); \
			double a = pop(); \
			push(a op b); \
		} while (false)

	for (;;) {

		#ifdef DEBUG_TRACE_EXECUTION
			printf("        ");
			for(Value* slot = vm.stack; slot < vm.stackTop; slot ++ ) {
				printf("[ ");
				printValue(*slot);
				printf(" ]");
			}
			printf("\n");
			disassembleInstruction(vm.chunk, (int) (vm.ip - vm.chunk -> code));
		#endif
		uint8_t instruction;
		switch (instruction = READ_BYTE()) {
			case OP_CONSTANT: {
				Value constant = READ_CONSTANT();
				printValue(constant);
				push(constant);
				printf("\n");
				break;
			}
			case OP_NEGATE: {
				// Little bit faster then push(-pop());
				// Used hyperfine (500_000 negations create chunk + disassemble + execute chunk)
				//
				// Using inplace modification:
				//     Time (mean ± σ):     534.5 ms ±   5.4 ms    [User: 508.4 ms, System: 36.3 ms]
				//     Range (min … max):   525.0 ms … 543.8 ms    10 runs
				// Using Pop:
				//     Time (mean ± σ):     549.7 ms ±   7.9 ms    [User: 526.0 ms, System: 34.5 ms]
				//     Range (min … max):   542.2 ms … 567.1 ms    10 runs


				*(vm.stackTop-1) = -(*(vm.stackTop-1));
				break;
			}
			case OP_ADD: BINARY_OP(+); break;
			case OP_SUBTRACT: BINARY_OP(-); break;
			case OP_MULTIPLY: BINARY_OP(*); break;
			case OP_DIVIDE: BINARY_OP(/); break;
			case OP_RETURN: {
				printValue(pop());
				return INTERPRET_OK;
			}
		}
	}

	#undef READ_BYTE
	#undef READ_CONSTANT
	#undef BINARY_OP

}

InterpretResult interpret(Chunk *chunk)
{
	vm.chunk = chunk;
	vm.ip = vm.chunk->code;
	return run();
}
