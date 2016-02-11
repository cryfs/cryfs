#include "CryConfigLoader.h"
#include "CryConfigFile.h"
#include <boost/filesystem.hpp>
#include <cpp-utils/random/Random.h>
#include <cpp-utils/logging/logging.h>

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
using std::shared_ptr;
using std::vector;
using std::string;
using std::function;
using std::shared_ptr;
using namespace cpputils::logging;

namespace cryfs {

CryConfigLoader::CryConfigLoader(shared_ptr<Console> console, RandomGenerator &keyGenerator, const SCryptSettings &scryptSettings, function<string()> askPasswordForExistingFilesystem, function<string()> askPasswordForNewFilesystem, const optional<string> &cipherFromCommandLine)
    : _creator(std::move(console), keyGenerator), _scryptSettings(scryptSettings),
      _askPasswordForExistingFilesystem(askPasswordForExistingFilesystem), _askPasswordForNewFilesystem(askPasswordForNewFilesystem),
      _cipherFromCommandLine(cipherFromCommandLine) {
}

optional<CryConfigFile> CryConfigLoader::_loadConfig(const bf::path &filename) {
  string password = _askPasswordForExistingFilesystem();
  std::cout << "Loading config file..." << std::flush;
  auto config = CryConfigFile::load(filename, password);
  if (config == none) {
    return none;
  }
  std::cout << "done" << std::endl;
  if (_cipherFromCommandLine != none && config->config()->Cipher() != *_cipherFromCommandLine) {
    throw std::runtime_error("Filesystem uses "+config->config()->Cipher()+" cipher and not "+*_cipherFromCommandLine+" as specified.");
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
  auto config = _creator.create(_cipherFromCommandLine);
  //TODO Ask confirmation if using insecure password (<8 characters)
  string password = _askPasswordForNewFilesystem();
  std::cout << "Creating config file..." << std::flush;
  auto result = CryConfigFile::create(filename, std::move(config), password, _scryptSettings);
  std::cout << "done" << std::endl;
  return result;
}


}
