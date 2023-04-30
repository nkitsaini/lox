#include <stdlib.h>
#include <memory.h>

void* reallocate(void* pointer, size_t oldSize, size_t newSize) {
	if (newSize == 0) {
		free(pointer);
		return NULL;
	}

	void* result = realloc(pointer, newSize);
	if (result == NULL) exit(1); // allocation failed due out of memory. Is it possible otherwise?
	return result;
}