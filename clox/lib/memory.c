#include "memory.h"
#include "backtrace.h"
#include "chunk.h"
#include "compiler.h"
#include "object.h"
#include "table.h"
#include "value.h"
#include "vm.h"
#include <memory.h>
#include <stdlib.h>

#ifdef DEBUG_LOG_GC
#include "debug.h"
#include <stdio.h>
#endif

#define GC_HEAD_GROW_FACTOR 2

void *reallocate(void *pointer, size_t oldSize, size_t newSize) {
  vm.bytesAllocated += newSize - oldSize;
  if (newSize > oldSize) {
#ifdef DEBUG_STRESS_GC
    collectGarbage();
#endif
    if (vm.bytesAllocated > vm.nextGC) {
      collectGarbage();
    }
  }
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);
  if (result == NULL)
    exit(1); // allocation failed due out of memory. Is it possible otherwise?
  return result;
}
void markObject(Obj *object) {
  if (object == NULL)
    return;
  if (object->isMarked)
    return;

#ifdef DEBUG_LOG_GC
  printf("%p mark ", (void *)object);
  printValue(OBJ_VAL(object));
  printf("\n");
#endif

  object->isMarked = true;

  if (vm.grayCapacity < vm.grayCount + 1) {
    vm.grayCapacity = GROW_CAPACITY(vm.grayCapacity);
    vm.grayStack =
        (Obj **)realloc(vm.grayStack, sizeof(Obj *) * vm.grayCapacity);
  }
  vm.grayStack[vm.grayCount++] = object;
}

void markValue(Value value) {
  if (IS_OBJ(value))
    markObject(AS_OBJ(value));
}

static void markArray(ValueArray *array) {
  for (int i = 0; i < array->count; i++) {
    markValue(array->values[i]);
  }
}

static void blackenObject(Obj *object) {
#ifdef DEBUG_LOG_GC
  printf("%p blacken ", (void *)object);
  printValue(OBJ_VAL(object));
  printf("\n");
#endif
  switch (object->type) {
  case OBJ_CLASS: {
    ObjClass *klass = (ObjClass *)object;
    // QUEST: Why not mark class itself?
    markObject((Obj *)klass->name);
    break;
  }
  case OBJ_CLOSURE: {
    ObjClosure *closure = (ObjClosure *)object;
    markObject((Obj *)closure->function);
    for (int i = 0; i < closure->upvalueCount; i++) {
      markObject((Obj *)closure->upvalues[i]);
    }
    break;
  }
  case OBJ_FUNCTION: {
    ObjFunction *function = (ObjFunction *)object;
    markObject((Obj *)function->name);
    markArray(&function->chunk.constants);
    break;
  }
  case OBJ_UPVALUE:
    markValue(((ObjUpvalue *)object)->closed);
    break;
  case OBJ_NATIVE:
  case OBJ_STRING:
    break;
  }
}

static void freeObject(Obj *object) {
#ifdef DEBUG_LOG_GC
  printf("%p free type %d\n", (void *)object, object->type);
  if (object->type == OBJ_STRING) {
    ObjString *ob = (ObjString *)object;
    printf("Freed string: '%.*s'\n", ob->length, ob->chars);
  }
#endif
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
  case OBJ_CLASS: {
    FREE(ObjClass, object);
  }
  }
}

static void markRoots() {
  for (Value *slot = vm.stack; slot < vm.stackTop; slot++) {
    markValue(*slot);
  }

  for (int i = 0; i < vm.frameCount; i++) {
    markObject((Obj *)vm.frames[i].closure);
  }

  for (ObjUpvalue *upvalue = vm.openUpvalues; upvalue != NULL;
       upvalue = upvalue->next) {
    markObject((Obj *)upvalue);
  }

  markTable(&vm.globals);
  markCompilerRoots();
}

static void traceReferences() {
  while (vm.grayCount > 0) {
    Obj *object = vm.grayStack[--vm.grayCount];
    blackenObject(object);
  }
}

static void sweep() {
  Obj *previous = NULL;
  Obj *object = vm.objects;
  while (object != NULL) {
    if (object->isMarked) {
      object->isMarked = false;
      previous = object;
      object = object->next;
    } else {
      Obj *unreached = object;
      object = object->next;
      if (previous != NULL) {
        previous->next = object;
      } else {
        vm.objects = object;
      }
      freeObject(unreached);
    }
  }
}

void collectGarbage() {
#ifdef DEBUG_LOG_GC
  printf("-- gc begin\n");
  print_trace();
  size_t before = vm.bytesAllocated;

  Value va;
  ObjString *str = tableFindString(&vm.strings, "a", 1, 3826002220);
  if (str == NULL) {
    printf(" A Global is not defined yet\n");
  } else {
    bool exists = tableGet(&vm.globals, str, &va);
    printf("done a search\n");
    if (exists) {
      printf("a Value is:");
      printValue(va);
      printf("\n");
    } else {
      printf("a Value is null\n");
    }
  }
#endif
  markRoots();
  traceReferences();
  tableRemoveWhite(&vm.strings);
  sweep();

  vm.nextGC = vm.bytesAllocated * GC_HEAD_GROW_FACTOR;

#ifdef DEBUG_LOG_GC
  printf("-- gc end\n");
  printf("   collected %zu bytes (from %zu to %zu) next at %zu\n",
         before - vm.bytesAllocated, before, vm.bytesAllocated, vm.nextGC);
#endif
}

void freeObjects() {
  Obj *object = vm.objects;
  while (object != NULL) {
    Obj *next = object->next;
    freeObject(object);
    object = next;
  }

  free(vm.grayStack);
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