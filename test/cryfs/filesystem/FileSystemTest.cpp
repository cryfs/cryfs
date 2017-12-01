#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <fspp/fstest/FsTest.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <cryfs/filesystem/CryDevice.h>
#include <cryfs/config/CryConfigLoader.h>
#include "../testutils/MockConsole.h"
#include "../testutils/TestWithFakeHomeDirectory.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::NoninteractiveConsole;
using fspp::Device;
using boost::none;
using std::make_shared;
using blockstore::inmemory::InMemoryBlockStore2;

using namespace cryfs;

class CryFsTestFixture: public FileSystemTestFixture, public TestWithMockConsole, public TestWithFakeHomeDirectory {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<InMemoryBlockStore2>();
    auto askPassword = [] {return "mypassword";};
    auto config = CryConfigLoader(make_shared<NoninteractiveConsole>(mockConsole()), Random::PseudoRandom(), SCrypt::TestSettings, askPassword, askPassword, none, none, none)
            .loadOrCreate(configFile.path()).value();
    return make_unique_ref<CryDevice>(std::move(config.configFile), std::move(blockStore), config.myClientId, false, false);
  }

  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
