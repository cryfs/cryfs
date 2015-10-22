#include <messmer/blockstore/implementations/testfake/FakeBlockStore.h>
#include <messmer/fspp/fstest/FsTest.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>

#include "../../src/filesystem/CryDevice.h"
#include "../testutils/MockConsole.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;

using fspp::Device;
using ::testing::Return;
using ::testing::_;

using blockstore::testfake::FakeBlockStore;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<FakeBlockStore>();
    auto config = CryConfigLoader(mockConsole(), cpputils::Random::PseudoRandom())
            .loadOrCreate(configFile.path());
    return make_unique_ref<CryDevice>(std::move(config), std::move(blockStore));
  }

  unique_ref<MockConsole> mockConsole() {
    auto console = make_unique_ref<MockConsole>();
    EXPECT_CALL(*console, ask(_, _)).WillRepeatedly(Return(0));
    EXPECT_CALL(*console, askYesNo(_)).WillRepeatedly(Return(true));
    return console;
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
