#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "CryFuse.h"

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  cryfs::CryDevice device(fusepp::path("/home/heinzi/cryfstest/root"));
  cryfs::CryFuse fuse(&device);
  fuse.run(argc, argv);
  return 0;
}
