#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>
#include <messmer/fspp/fstest/FsTest.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>

#include "../../src/filesystem/CryDevice.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;

using fspp::Device;

using blockstore::testfake::FakeBlockStore;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<FakeBlockStore>();
    auto config = CryConfigLoader().loadOrCreateForTest(configFile.path());
    return make_unique_ref<CryDevice>(std::move(config), std::move(blockStore));
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
