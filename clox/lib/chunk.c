#include "chunk.h"

void initChunk(Chunk* chunk) {
	chunk->capacity = 0;
	chunk->count = 0;
	chunk->code = NULL;
	chunk->lines = NULL;
	chunk->line_vec_capacity = 0;
	chunk->line_vec_count = 0;
	initValueArray(&chunk->constants);
}

void addLine(Chunk* chunk, int line) {
	// printf("===== Add Line %d", line);
	if (chunk->line_vec_count != 0 && chunk->lines[chunk->line_vec_count-1] == line) {
		chunk->lines[chunk->line_vec_count-2]++;
		return;
	}

	if (chunk->line_vec_count > chunk->line_vec_capacity -2) { // -1, -2 both will work fine here. since we always increase by atleast two.
		int oldCapacity = chunk -> line_vec_capacity;

		// Due to GROW_CAPACITY guarantees, we'll atleast increase capacity by 2.
		// Which is what we need.
		chunk -> line_vec_capacity = GROW_CAPACITY(oldCapacity);
		chunk -> lines = GROW_ARRAY(int, chunk -> lines, oldCapacity, chunk -> line_vec_capacity);
	}

	chunk->lines[chunk -> line_vec_count] = 1;
	chunk->lines[chunk -> line_vec_count + 1] = line;
	chunk -> line_vec_count += 2;
}

int getLine(Chunk* chunk, int idx) {
	int org_idx = idx;
	idx++;
	for (int block_idx =1; chunk->line_vec_count >= block_idx*2; block_idx++) {
		if (chunk->lines[(block_idx*2) - 2] >= idx) {
			// Belongs to current block
			// printf("||Returning get line %d for %d || ", chunk->lines[(block_idx*2) - 1], org_idx);
			return chunk->lines[(block_idx*2) - 1];
		}
		idx -= chunk->lines[(block_idx*2) - 2]; // move to next block
	}
	// There's no more blocks to check
	return -1;
}

void writeChunk(Chunk* chunk, uint8_t byte, int line) {
	if (chunk->count > chunk->capacity -1) {
		int oldCapacity = chunk -> capacity;
		chunk->capacity = GROW_CAPACITY(oldCapacity);
		chunk->code = GROW_ARRAY(uint8_t, chunk->code, oldCapacity, chunk->capacity);
	}

	chunk->code[chunk->count] = byte;
	addLine(chunk, line);
	chunk->count +=1;
}

int addConstant(Chunk* chunk, Value value) {
	writeValueArray(&chunk->constants, value);
	return chunk->constants.count - 1;
}

void addConstantAddress(Chunk* chunk, int address, int line) {

	if (address < 256) { // can fit in regular OP_CONSTANT (8 bit)
		writeChunk(chunk, OP_CONSTANT, line);
		writeChunk(chunk, address, line);
		return;
	}

	writeChunk(chunk, OP_CONSTANT_LONG, line);
	writeChunk(chunk, address >> 16, line);
	writeChunk(chunk, (address >> 8) & ((1 << 8) - 1), line);
	writeChunk(chunk, (address) & ((1 << 8) - 1), line);
	// writeChunk(chunk, address >> 16, line);
	// writeChunk(&chunk, address >> 8, line);
	// writeChunk(&chunk, address >> 16, line);
	return;
}


void freeChunk(Chunk* chunk) {
	FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
	FREE_ARRAY(int, chunk->lines, chunk->capacity);
	freeValueArray(&chunk->constants);
	initChunk(chunk);
}