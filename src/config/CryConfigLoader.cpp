#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <messmer/cpp-utils/random/Random.h>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::IOStreamConsole;
using boost::optional;
using boost::none;
using std::vector;
using std::string;

namespace cryfs {

CryConfigLoader::CryConfigLoader(unique_ref<Console> console, cpputils::RandomGenerator &keyGenerator)
        : _creator(std::move(console), keyGenerator) {}

CryConfigFile CryConfigLoader::loadOrCreate(const bf::path &filename) {
  auto config = CryConfigFile::load(filename);
  if (config != none) {
    return std::move(*config);
  }
  return CryConfigFile::create(filename, _creator.create());
}

}
