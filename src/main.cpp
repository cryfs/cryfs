#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "fusepp/fusebindings/Fuse.h"
#include "fusepp/impl/FilesystemImpl.h"
#include "cryfs_lib/CryDevice.h"

namespace bf = boost::filesystem;

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  cryfs::CryDevice device(bf::path("/home/heinzi/cryfstest/root"));
  fusepp::FilesystemImpl fsimpl(&device);
  fusepp::fusebindings::Fuse fuse(&fsimpl);
  fuse.run(argc, argv);
  return 0;
}
