#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <fspp/fstest/FsTest.h>
#include <cpp-utils/tempfile/TempFile.h>

#include <cryfs/filesystem/CryDevice.h>
#include <cryfs/config/CryConfigLoader.h>
#include "../testutils/MockConsole.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Random;
using cpputils::SCrypt;
using fspp::Device;
using ::testing::Return;
using ::testing::_;
using boost::none;

using blockstore::testfake::FakeBlockStore;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture, public TestWithMockConsole {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<FakeBlockStore>();
    auto askPassword = [] {return "mypassword";};
    auto config = CryConfigLoader(mockConsole(), Random::PseudoRandom(), SCrypt::TestSettings, askPassword, askPassword, none, none, true)
            .loadOrCreate(configFile.path()).value();
    return make_unique_ref<CryDevice>(std::move(config), std::move(blockStore));
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
