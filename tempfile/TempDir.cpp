#include "TempDir.h"

namespace bf = boost::filesystem;

namespace cpputils {

TempDir::TempDir()
  : _path(bf::unique_path(bf::temp_directory_path() / "%%%%-%%%%-%%%%-%%%%")), _existing(true) {
  bf::create_directory(_path);
}

TempDir::~TempDir() {
  remove();
}

void TempDir::remove() {
  if (_existing) {
    bf::remove_all(_path);
    _existing = false;
  }
}

const bf::path &TempDir::path() const {
  return _path;
}

}
