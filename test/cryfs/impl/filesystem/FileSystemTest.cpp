#include "fspp/fstest/testutils/FileSystemTest.h"
#include "../testutils/MockConsole.h"
#include "../testutils/TestWithFakeHomeDirectory.h"
#include "cpp-utils/crypto/kdf/Scrypt.h"
#include "cpp-utils/pointer/unique_ref.h"
#include "cpp-utils/tempfile/TempDir.h"
#include "cryfs/impl/localstate/LocalStateDir.h"
#include "fspp/fs_interface/Device.h"
#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <boost/none.hpp>
#include <cpp-utils/io/NoninteractiveConsole.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cryfs/impl/config/CryConfigLoader.h>
#include <cryfs/impl/config/CryPresetPasswordBasedKeyProvider.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <fspp/fstest/FsTest.h>
#include <gtest/gtest.h>
#include <utility>

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Random;
using cpputils::SCrypt;
using cpputils::NoninteractiveConsole;
using fspp::Device;
using boost::none;
using std::make_shared;
using blockstore::inmemory::InMemoryBlockStore2;
using cryfs::CryPresetPasswordBasedKeyProvider;

using namespace cryfs;

namespace {

auto failOnIntegrityViolation() {
  return [] {
    EXPECT_TRUE(false);
  };
}

class CryFsTestFixture: public FileSystemTestFixture, public TestWithMockConsole, public TestWithFakeHomeDirectory {
public:
  CryFsTestFixture()
  // Don't create config tempfile yet
  : tempLocalStateDir(), localStateDir(tempLocalStateDir.path()), configFile(false) {}

  unique_ref<Device> createDevice() override {
    auto blockStore = cpputils::make_unique_ref<InMemoryBlockStore2>();
    auto _console = make_shared<NoninteractiveConsole>(mockConsole());
    auto keyProvider = make_unique_ref<CryPresetPasswordBasedKeyProvider>("mypassword", make_unique_ref<SCrypt>(SCrypt::TestSettings));
    auto config = CryConfigLoader(_console, Random::PseudoRandom(), std::move(keyProvider), localStateDir, none, none, none)
            .loadOrCreate(configFile.path(), false, false).right();
    return make_unique_ref<CryDevice>(std::move(config.configFile), std::move(blockStore), localStateDir, config.myClientId, false, false, failOnIntegrityViolation());
  }

  cpputils::TempDir tempLocalStateDir;
  LocalStateDir localStateDir;
  cpputils::TempFile configFile;
};

FSPP_ADD_FILESYTEM_TESTS(CryFS, CryFsTestFixture);
}
