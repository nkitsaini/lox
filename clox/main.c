#include "lib/common.h"
#include "lib/chunk.h"
#include "lib/debug.h"
#include <stdlib.h>

int main(int argc, char* argv[]) {
	int* a = NULL;
	realloc(a, 40);
	return 0;
	// Chunk chunk;
	// initChunk(&chunk);
	// writeChunk(&chunk, OP_RETURN, 1);

	// int constant = addConstant(&chunk, 1.2);
	// writeChunk(&chunk, OP_CONSTANT, 2);
	// printf("Constant Add: %d", constant);
	// writeChunk(&chunk, constant, 2);

	// disassembleChunk(&chunk, "test chunk");
	// freeChunk(&chunk);

	// printf("Hey");
	// return 0;
}
