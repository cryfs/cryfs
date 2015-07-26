#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>

#include "messmer/fspp/fuse/Fuse.h"
#include "messmer/fspp/impl/FilesystemImpl.h"
#include "CryDevice.h"
#include "CryConfigLoader.h"

namespace bf = boost::filesystem;

using blockstore::ondisk::OnDiskBlockStore;
using blockstore::inmemory::InMemoryBlockStore;

using cpputils::make_unique_ref;

int main (int argc, char *argv[])
{
  auto blockStore = make_unique_ref<OnDiskBlockStore>(bf::path("/home/heinzi/cryfstest/root"));
  auto config = cryfs::CryConfigLoader().loadOrCreate(bf::path("/home/heinzi/cryfstest/config.json"));
  cryfs::CryDevice device(std::move(config), std::move(blockStore));
  fspp::FilesystemImpl fsimpl(&device);
  fspp::fuse::Fuse fuse(&fsimpl);
  fuse.run(argc, argv);
  return 0;
}
