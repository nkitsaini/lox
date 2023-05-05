#include "lib/common.h"
#include "lib/chunk.h"
#include "lib/debug.h"
#include <stdlib.h>

int main(int argc, char* argv[]) {
	Chunk chunk;
	initChunk(&chunk);
	writeChunk(&chunk, OP_RETURN, 1);

	for (int i = 0; i< 300; i ++ ) {
		int constant = addConstant(&chunk, 1.2);
		// writeChunk(&chunk, OP_CONSTANT, 2);
		// printf("Constant Add: %d", constant);
		// writeChunk(&chunk, constant, 2);
		printf("writing\n");
		addConstantAddress(&chunk, constant, 2);
		printf("Wrote\n");

	}

	disassembleChunk(&chunk, "test chunk");
	freeChunk(&chunk);

	printf("Hey");
	return 0;
}
