#include "TempFile.h"

#include <fstream>

namespace bf = boost::filesystem;
using std::ofstream;

namespace cpputils {

TempFile::TempFile(const bf::path &path, bool create)
  : _path(path) {
  if (create) {
    ofstream file(_path.c_str());
  }
}

TempFile::TempFile(bool create)
  : TempFile(bf::unique_path(bf::temp_directory_path() / "%%%%-%%%%-%%%%-%%%%"), create) {
}

TempFile::~TempFile() {
  if (exists()) {
    bf::remove(_path);
  }
}

bool TempFile::exists() const {
  return bf::exists(_path);
}

const bf::path &TempFile::path() const {
  return _path;
}

}
