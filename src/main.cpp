// A simple program that computes the square root of a number
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "cryfs_lib/CryFuse.h"

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  cryfs::CryFuse fuse;
  fuse.run(argc, argv);
  return 0;
}
