#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <messmer/cpp-utils/random/Random.h>
#include <messmer/cpp-utils/logging/logging.h>

namespace bf = boost::filesystem;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Console;
using cpputils::IOStreamConsole;
using cpputils::Random;
using cpputils::RandomGenerator;
using cpputils::SCryptSettings;
using boost::optional;
using boost::none;
using std::vector;
using std::string;
using std::function;
using std::shared_ptr;
using namespace cpputils::logging;

namespace cryfs {

CryConfigLoader::CryConfigLoader(unique_ref<Console> console, RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, function<string()> askPasswordForExistingFilesystem, function<string()> askPasswordForNewFilesystem, const optional<string> &cipher)
    : _creator(std::move(console), keyGenerator), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipher(cipher) {
}

optional<CryConfigFile> CryConfigLoader::_loadConfig(const bf::path &filename) {
  string password = _askPasswordForExistingFilesystem();
  std::cout << "Loading config file..." << std::flush;
  auto config = CryConfigFile::load(filename, password);
  if (config == none) {
    LOG(ERROR) << "Could not load config file. Wrong password?";
    return none;
  }
  std::cout << "done" << std::endl;
  if (_cipher != none && config->config()->Cipher() != *_cipher) {
    throw std::runtime_error("Filesystem uses "+config->config()->Cipher()+" cipher and not "+*_cipher+" as specified.");
  }
  return std::move(*config);
}

optional<CryConfigFile> CryConfigLoader::loadOrCreate(const bf::path &filename) {
  if (bf::exists(filename)) {
    return _loadConfig(filename);
  } else {
    return _createConfig(filename);
  }
}

CryConfigFile CryConfigLoader::_createConfig(const bf::path &filename) {
  auto config = _creator.create(_cipher);
  //TODO Ask confirmation if using insecure password (<8 characters)
  string password = _askPasswordForNewFilesystem();
  std::cout << "Creating config file..." << std::flush;
  auto result = CryConfigFile::create(filename, std::move(config), password, _scryptSettings);
  std::cout << "done" << std::endl;
  return result;
}


}
