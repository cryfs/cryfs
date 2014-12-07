#include <cmath>
#include <cstdio>
#include <cstdlib>
#include "buildconfig/BuildConfig.h"

#include "fspp/fuse/Fuse.h"
#include "fspp/impl/FilesystemImpl.h"
#include "copyfs/CopyDevice.h"
#include "cryfs_lib/CryDevice.h"

namespace bf = boost::filesystem;

int main (int argc, char *argv[])
{
  printf("Version: %d\n", buildconfig::VERSION::MAJOR);
  copyfs::CopyDevice device(bf::path("/home/heinzi/cryfstest/root"));
  fspp::FilesystemImpl fsimpl(&device);
  fspp::fuse::Fuse fuse(&fsimpl);
  fuse.run(argc, argv);
  return 0;
}
