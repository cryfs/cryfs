#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <messmer/cpp-utils/random/Random.h>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::IOStreamConsole;
using cpputils::Random;
using cpputils::RandomGenerator;
using boost::optional;
using boost::none;
using std::vector;
using std::string;
using std::function;

namespace cryfs {

CryConfigLoader::CryConfigLoader(unique_ref<Console> console, RandomGenerator &keyGenerator, function<string()> askPassword)
    : _creator(std::move(console), keyGenerator), _askPassword(askPassword) {
}

CryConfigFile CryConfigLoader::loadOrCreate(const bf::path &filename) {
  if (bf::exists(filename)) {
    return _loadConfig(filename);
  } else {
    return _createConfig(filename);
  }
}

CryConfigFile CryConfigLoader::_loadConfig(const bf::path &filename) {
  string password = _askPassword();
  auto config = CryConfigFile::load(filename, password);
  if (config == none) {
    std::cerr << "Could not load config file. Wrong password?" << std::endl;
    exit(1);
  }
  return std::move(*config);
}

CryConfigFile CryConfigLoader::_createConfig(const bf::path &filename) {
  auto config = _creator.create();
  //TODO Ask confirmation if using insecure password (<8 characters)
  string password = _askPassword();
  return CryConfigFile::create(filename, std::move(config), password);
}

}
