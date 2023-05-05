#include "lib/common.h"
#include "lib/chunk.h"
#include "lib/debug.h"
#include "lib/vm.h"
#include <stdlib.h>

int main(int argc, char* argv[]) {
	initVM();
	Chunk chunk;
	initChunk(&chunk);

	for (int i = 0; i< 3; i ++ ) {
		int constant = addConstant(&chunk, 1.2);
		// writeChunk(&chunk, OP_CONSTANT, 2);
		// printf("Constant Add: %d", constant);
		// writeChunk(&chunk, constant, 2);
		printf("writing\n");
		addConstantAddress(&chunk, constant, 1);
		printf("Wrote\n");

	}
	for (int i = 0; i < 500000; i++ ) {
		writeChunk(&chunk, OP_NEGATE, 2);
	}

	writeChunk(&chunk, OP_ADD, 2);
	int constant = addConstant(&chunk, 33);
	addConstantAddress(&chunk, constant, 2);
	writeChunk(&chunk, OP_SUBTRACT, 2);
	writeChunk(&chunk, OP_NEGATE, 2);
	writeChunk(&chunk, OP_RETURN, 2);
	disassembleChunk(&chunk, "test chunk");
	printf("========= Interpreting ============");
	interpret(&chunk);
	freeVM();
	freeChunk(&chunk);

	return 0;
}
