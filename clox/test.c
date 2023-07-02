#include "lib/chunk.h"
#include "lib/common.h"
#include "lib/debug.h"
#include "lib/greatest.h"

/* A test runs various assertions, then calls PASS(), FAIL(), or SKIP(). */
TEST should_serve_as_documentation(void) {
  int x = 1;
  /* Compare, with an automatic "1 != x" failure message */
  ASSERT_EQ(1, x);

  /* Compare, with a custom failure message */
  ASSERT_EQm("Yikes, x doesn't equal 1", 1, x);

  /* Compare, and if they differ, print both values,
   * formatted like `printf("Expected: %d\nGot: %d\n", 1, x);` */
  ASSERT_EQ_FMT(1, x, "%d");
  PASS();
}

TEST chunk_run_store_lines(void) {
  Chunk chunk;
  initChunk(&chunk);

  addLine(&chunk, 1);  // 1
  addLine(&chunk, 1);  // 2
  addLine(&chunk, 2);  // 3
  addLine(&chunk, 3);  // 4
  addLine(&chunk, 4);  // 5
  addLine(&chunk, 4);  // 6
  addLine(&chunk, 4);  // 7
  addLine(&chunk, 5);  // 8
  addLine(&chunk, 6);  // 9
  ASSERT_EQ(1, getLine(&chunk, 0));
  ASSERT_EQ(1, getLine(&chunk, 1));
  ASSERT_EQ(2, getLine(&chunk, 2));
  ASSERT_EQ(3, getLine(&chunk, 3));
  ASSERT_EQ(4, getLine(&chunk, 4));
  ASSERT_EQ(4, getLine(&chunk, 5));
  ASSERT_EQ(4, getLine(&chunk, 6));
  ASSERT_EQ(5, getLine(&chunk, 7));
  ASSERT_EQ(6, getLine(&chunk, 8));

  // Out of bound
  ASSERT_EQ(-1, getLine(&chunk, 10));
  PASS();
}

/* Suites can group multiple tests with common setup. */
SUITE(the_suite) {
  RUN_TEST(should_serve_as_documentation);
  RUN_TEST(chunk_run_store_lines);
}

/* Add definitions that need to be in the test runner's main file. */
GREATEST_MAIN_DEFS();

int main(int argc, char **argv) {
  GREATEST_MAIN_BEGIN(); /* command-line options, initialization. */

  /* Individual tests can be run directly in main, outside of suites. */
  /* RUN_TEST(x_should_equal_1); */

  /* Tests can also be gathered into test suites. */
  RUN_SUITE(the_suite);

  GREATEST_MAIN_END(); /* display results */
}