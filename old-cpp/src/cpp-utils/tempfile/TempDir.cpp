#include "TempDir.h"
#include "../logging/logging.h"

namespace bf = boost::filesystem;
using namespace cpputils::logging;

namespace cpputils {

TempDir::TempDir()
  : _path(bf::unique_path(bf::temp_directory_path() / "%%%%-%%%%-%%%%-%%%%")) {
  bf::create_directory(_path);
}

TempDir::~TempDir() {
  remove();
}

void TempDir::remove() {
  try {
    if (bf::exists(_path)) {
      bf::remove_all(_path);
    }
  } catch (const boost::filesystem::filesystem_error &e) {
    LOG(ERR, "Could not delete tempfile.");
  }
}

const bf::path &TempDir::path() const {
  return _path;
}

}
