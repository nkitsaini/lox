#include <unistd.h> // for usleep

void sleep_ms(int milliseconds) { // non cross-platform sleep function
  if (milliseconds >= 1000)
    sleep(milliseconds / 1000);
  usleep((milliseconds % 1000) * 1000);
}