#pragma once
#ifndef MESSMER_CPPUTILS_TEMPFILE_TEMPDIR_H_
#define MESSMER_CPPUTILS_TEMPFILE_TEMPDIR_H_

#include <boost/filesystem.hpp>
#include "../macros.h"

namespace cpputils {

class TempDir {
public:
  TempDir();
  virtual ~TempDir();
  const boost::filesystem::path &path() const;

private:
  const boost::filesystem::path _path;

  DISALLOW_COPY_AND_ASSIGN(TempDir);
};

}

#endif
