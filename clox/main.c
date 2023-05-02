#include "common.h"
#include "chunk.h"
#include "debug.h"

int main(int argc, const* argv[]) {
	Chunk chunk;
	initChunk(&chunk);
	writeChunk(&chunk, OP_RETURN, 1);

	int constant = addConstant(&chunk, 1.2);
	writeChunk(&chunk, OP_CONSTANT, 2);
	printf("Constant Add: %d", constant);
	writeChunk(&chunk, constant, 2);

	disassembleChunk(&chunk, "test chunk");
	freeChunk(&chunk);

	printf("Hey");
	return 0;
}
