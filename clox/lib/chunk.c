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

void writeChunk(Chunk* chunk, uint8_t byte, int line) {
	if (chunk->count > chunk->capacity -1) {
		int oldCapacity = chunk -> capacity;
		chunk->capacity = GROW_CAPACITY(oldCapacity);
		chunk->code = GROW_ARRAY(uint8_t, chunk->code, oldCapacity, chunk->capacity);
		chunk->lines = GROW_ARRAY(int, chunk->lines, oldCapacity, chunk->capacity);
	}

	chunk->code[chunk->count] = byte;
	chunk->lines[chunk->count] = line;
	chunk->count +=1;
}

int getLine(Chunk* chunk, int token_idx) {
	
}

int addConstant(Chunk* chunk, Value value) {
	writeValueArray(&chunk->constants, value);
	return chunk->constants.count - 1;
}

void freeChunk(Chunk* chunk) {
	FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
	FREE_ARRAY(int, chunk->lines, chunk->capacity);
	freeValueArray(&chunk->constants);
	initChunk(chunk);
}