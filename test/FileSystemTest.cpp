#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>
#include <messmer/fspp/fstest/FsTest.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>

#include "../src/CryDevice.h"

using std::unique_ptr;
using std::make_unique;

using fspp::Device;

using blockstore::testfake::FakeBlockStore;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ptr<Device> createDevice() override {
    auto blockStore = make_unique<FakeBlockStore>();
    auto config = make_unique<CryConfig>(configFile.path());
    return make_unique<CryDevice>(std::move(config), std::move(blockStore));
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
