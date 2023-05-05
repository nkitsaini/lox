#ifndef clox_chunk_h
#define clox_chunk_h

#include "common.h"
#include "memory.h"
#include "value.h"

typedef enum {
	OP_CONSTANT,
	OP_CONSTANT_LONG,
	OP_RETURN,
} OpCode;

typedef struct {
	int count;
	int capacity;
	uint8_t* code;
	int line_vec_count;
	int line_vec_capacity;
	int* lines;
	ValueArray constants;
} Chunk;

void initChunk(Chunk* chunk);
void writeChunk(Chunk* chunk, uint8_t byte, int line);
void addLine(Chunk* chunk, int line);
int getLine(Chunk* chunk, int idx);
int addConstant(Chunk* chunk, Value value);
void addConstantAddress(Chunk* chunk, int address, int line);
void freeChunk(Chunk* chunk);

#endif
