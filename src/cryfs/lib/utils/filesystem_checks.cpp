#include "filesystem_checks.h"
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <boost/optional.hpp>
#include <fstream>

namespace bf = boost::filesystem;
using boost::optional;
using boost::none;
using std::shared_ptr;
using std::make_shared;
using std::ifstream;
using std::ofstream;
using cpputils::TempFile;

namespace {

optional<shared_ptr<TempFile>> _try_write_file(const bf::path &dir) {
  auto path = dir / "tempfile";
  try {
    return make_shared<TempFile>(path);
  } catch (const std::runtime_error &e) {
    return none;
  }
}

bool _check_dir_readable(const bf::path &dir, shared_ptr<TempFile> tempfile) {
  ASSERT(bf::equivalent(dir, tempfile->path().parent_path()), "This function should be called with a file inside the directory");
  try {
    bool found = false;
    bf::directory_iterator end;
    for (auto iter = bf::directory_iterator(dir); iter != end; ++iter) {
      if (bf::equivalent(*iter, tempfile->path())) {
        found = true;
      }
    }
    if (!found) {
      return false; // The dir doesn't seem to contain the file
    }
    return true;
  } catch (const bf::filesystem_error &e) {
    return false; // Reading from the directory failed
  }
}

}

namespace filesystem_checks {

bool check_dir_accessible(const bf::path &dir) {
  ASSERT(bf::exists(dir), "This should be checked before calling this function");
  if (!bf::is_directory(dir)) {
    return false;
  }
  optional <shared_ptr<TempFile>> file = _try_write_file(dir);
  if (none == file) {
    return false; // Couldn't write to dir
  }
  return _check_dir_readable(dir, *file);
}

bool check_file_readable(const bf::path &file) {
  ASSERT(bf::exists(file), "This should be checked before calling this function");
  if (!bf::is_regular_file(file)) {
    return false;
  }

  return ifstream(file.native().c_str()).good();
}

bool check_file_appendable(const bf::path &file) {
  ASSERT(bf::exists(file), "This should be checked before calling this function");
  if (!bf::is_regular_file(file)) {
    return false;
  }

  return ofstream(file.native().c_str(), std::ios_base::app).good();
}

}
