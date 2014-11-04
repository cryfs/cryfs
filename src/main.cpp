// A simple program that computes the square root of a number
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "cryfs_lib/CryDevice.h"

int main ()
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  cryfs::CryDevice device;
  #ifdef NDEBUG
  printf("Release build");
  #else
  printf("Debug build");
  #endif
  return 0;
}
