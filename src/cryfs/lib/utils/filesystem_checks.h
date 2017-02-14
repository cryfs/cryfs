#pragma once
#ifndef CRYFS_CRYFS_UTILS_FILESYSTEMCHECKS_H
#define CRYFS_CRYFS_UTILS_FILESYSTEMCHECKS_H

#include <boost/filesystem/path.hpp>

namespace filesystem_checks {
  bool check_dir_accessible(const boost::filesystem::path &dir);
  bool check_file_readable(const boost::filesystem::path &file);
  bool check_file_appendable(const boost::filesystem::path &file);
}

#endif
