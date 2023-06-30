#include "memory.h"
#include "chunk.h"
#include "object.h"
#include "value.h"
#include "vm.h"
#include <memory.h>
#include <stdlib.h>

void *reallocate(void *pointer, size_t oldSize, size_t newSize) {
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);
  if (result == NULL)
    exit(1); // allocation failed due out of memory. Is it possible otherwise?
  return result;
}

static void freeObject(Obj *object) {
  switch (object->type) {
  case OBJ_STRING: {
    ObjString *string = (ObjString *)object;
    FREE_ARRAY(char, string->chars, string->length + 1);
    FREE(ObjString, object);
    break;
  }
  case OBJ_FUNCTION: {
    ObjFunction *function = (ObjFunction *)object;
    freeChunk(&function->chunk);
    FREE(ObjFunction, object);
    break;
  }
  case OBJ_CLOSURE: {
    ObjClosure *closure = (ObjClosure *)object;
    FREE_ARRAY(ObjUpvalue *, closure->upvalues, closure->upvalueCount);
    FREE(ObjClosure, object);
    break;
  }
  case OBJ_NATIVE: {
    FREE(ObjNative, object);
    break;
  }
  case OBJ_UPVALUE: {
    FREE(ObjUpvalue, object);
  }
  }
}

void freeObjects() {
  Obj *object = vm.objects;
  while (object != NULL) {
    Obj *next = object->next;
    freeObject(object);
    object = next;
  }
}

// void* ptr = NULL;
// int ptr_initialized = 0;

// typedef struct {

// } MemoryFragements;

// /*
// -----------------------------------------------------------------------------------------------

// free(x):
// 	if free_memory[end]:
// 		free_memory[x] = free_memory[end]
// 		delete free_memory[end]
// 	else:
// 		free_memory[x] = end

// malloc(ptr, size):
// 	end = allocated_memroy[ptr]
// 	if free_memory[end] - end >= size:
// 		free_memory[free_memory[end]] = free_memory[end]
// 		allocated_memory[ptr] += size
// 		delete free_memory[end]
// 		return ptr
// 	else:
// 		new_start = free_memory.find((start, end) => end - start >=
// size) 		allocated_memory[new_start] = new_start + size
// copy(ptr, end, new_start, new_start + size)

// 		free_memory[new_start + size] = free_memory[new_start];
// 		delete free_memory[new_start];
// 		return new_start;

// */

// static void free_hardcore(void* pointer) {

// }

// static void* realloc_hardcore(void* pointer, size_t newSize) {

// }

// void* reallocate_hardcore_mode(void* pointer, size_t oldSize, size_t newSize)
// { 	if (ptr_initialized == 0) { 		ptr = malloc(1024 * 1024 * 10);
// // 10 MB
// 	}

// 	if (newSize == 0) {
// 		free_hardcore(pointer);
// 		return NULL;
// 	}

// 	void* result = realloc_hardcore(pointer, newSize);
// 	if (result == NULL) exit(1); // allocation failed due out of memory. Is
// it possible otherwise? 	return result;
// }