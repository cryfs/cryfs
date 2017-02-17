#pragma once
#ifndef CRYFS_CRYFS_UNMOUNTER_H
#define CRYFS_CRYFS_UNMOUNTER_H

#include "../cryfs.h"
#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>

namespace cryfs {

class cryfs_unmounter final {
public:
  static cryfs_status unmount(const boost::filesystem::path &mountdir);

private:
  cryfs_unmounter() = delete;

  DISALLOW_COPY_AND_ASSIGN(cryfs_unmounter);
};

}

#endif
