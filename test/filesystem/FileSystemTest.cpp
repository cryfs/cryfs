#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>
#include <messmer/fspp/fstest/FsTest.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>

#include "../../src/filesystem/CryDevice.h"
#include "../../src/config/CryConfigLoader.h"
#include "../testutils/MockConsole.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Random;

using fspp::Device;
using ::testing::Return;
using ::testing::_;

using blockstore::testfake::FakeBlockStore;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture, public TestWithMockConsole {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<FakeBlockStore>();
    auto config = CryConfigLoader(mockConsole(), Random::PseudoRandom(), [] {return "mypassword";})
            .loadOrCreate(configFile.path());
    return make_unique_ref<CryDevice>(std::move(config), std::move(blockStore));
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
