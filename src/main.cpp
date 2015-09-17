#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>

#include "messmer/fspp/fuse/Fuse.h"
#include "messmer/fspp/impl/FilesystemImpl.h"
#include "filesystem/CryDevice.h"
#include "config/CryConfigLoader.h"

#include "version/VersionHandler.h"

namespace bf = boost::filesystem;

using blockstore::ondisk::OnDiskBlockStore;
using blockstore::inmemory::InMemoryBlockStore;

using cpputils::make_unique_ref;
using std::cout;
using std::endl;

int main (int argc, char *argv[])
{
  cout << "CryFS Version "<<version::VERSION.toString() << endl;
  if (version::COMMITS_SINCE_TAG != 0) {
    cout << "WARNING! This is a development version based on git commit " << version::GIT_COMMIT_ID << ". Please do not use in production!" << endl;
  } else if (!version::VERSION.is_stable()) {
    cout << "WARNING! This is an experimental version. Please backup your data frequently!" << endl;
  }
  cout << endl;
  auto blockStore = make_unique_ref<OnDiskBlockStore>(bf::path("/home/heinzi/cryfstest/root"));
  auto config = cryfs::CryConfigLoader().loadOrCreate(bf::path("/home/heinzi/cryfstest/config.json"));
  cryfs::CryDevice device(std::move(config), std::move(blockStore));
  fspp::FilesystemImpl fsimpl(&device);
  fspp::fuse::Fuse fuse(&fsimpl);
  fuse.run(argc, argv);
  return 0;
}
