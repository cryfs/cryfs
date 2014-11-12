#ifndef CRYFS_LIB_CRYOPENDIR_H_
#define CRYFS_LIB_CRYOPENDIR_H_

#include <boost/filesystem.hpp>
#include <memory>
#include <vector>
#include <string>
#include <dirent.h>

#include "utils/macros.h"

namespace cryfs {
class CryDevice;

namespace bf = boost::filesystem;

class CryOpenDir {
public:
  CryOpenDir(const CryDevice *device, const bf::path &path);
  virtual ~CryOpenDir();

  std::unique_ptr<std::vector<std::string>> readdir() const;
private:
  DIR *_dir;

  DISALLOW_COPY_AND_ASSIGN(CryOpenDir);
};

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYOPENDIR_H_ */
