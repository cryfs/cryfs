// A simple program that computes the square root of a number
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "cryfs_lib/CryDevice.h"

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  Fuse fuse;
  fuse.run(argc, argv);
  return 0;
}
