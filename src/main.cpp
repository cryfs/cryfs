// A simple program that computes the square root of a number
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "cryfs_lib/CryFuse.h"

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  cryfs::CryDevice device(fusepp::path("/mnt/root"));
  cryfs::CryFuse fuse(&device);
  fuse.run(argc, argv);
  return 0;
}
