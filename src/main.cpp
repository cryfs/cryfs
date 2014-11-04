// A simple program that computes the square root of a number
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

int main ()
{
  printf("Version: %d", buildconfig::VERSION::MAJOR);
  #ifdef NDEBUG
  printf("Release build");
  #else
  printf("Debug build");
  #endif
  return 0;
}
