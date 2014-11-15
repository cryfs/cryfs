#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "CryFuse.h"

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  fusepp::FuseDevice device(fusepp::path("/home/heinzi/cryfstest/root"));
  fusepp::CryFuse fuse(&device);
  fuse.run(argc, argv);
  return 0;
}
