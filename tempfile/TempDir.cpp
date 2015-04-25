#include "TempDir.h"

namespace bf = boost::filesystem;

namespace cpputils {

TempDir::TempDir()
  : _path(bf::unique_path(bf::temp_directory_path() / "%%%%-%%%%-%%%%-%%%%")) {
  bf::create_directory(_path);
}

TempDir::~TempDir() {
  bf::remove_all(_path);
}

const bf::path &TempDir::path() const {
  return _path;
}

}
