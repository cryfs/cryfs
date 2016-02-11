#pragma once
#ifndef MESSMER_CPPUTILS_TEMPFILE_TEMPDIR_H_
#define MESSMER_CPPUTILS_TEMPFILE_TEMPDIR_H_

#include <boost/filesystem.hpp>
#include "../macros.h"

namespace cpputils {

class TempDir final {
public:
  TempDir();
  ~TempDir();
  const boost::filesystem::path &path() const;
  void remove();

private:
  const boost::filesystem::path _path;

  DISALLOW_COPY_AND_ASSIGN(TempDir);
};

}

#endif
