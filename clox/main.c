#include "common.h"
#include "chunk.h"
#include "debug.h"

int main(int argc, const* argv[]) {
	Chunk chunk;
	initChunk(&chunk);
	writeChunk(&chunk, OP_RETURN);

	disassembleChunk(&chunk, "test chunk");
	freeChunk(&chunk);

	printf("Hey");
	return 0;
}
