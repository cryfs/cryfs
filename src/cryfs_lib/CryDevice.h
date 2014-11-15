#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>

#include "fspp/utils/macros.h"

namespace cryfs {

namespace bf = boost::filesystem;

class CryDevice: public fspp::Device {
public:
  CryDevice(const bf::path &rootdir);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  const bf::path &RootDir() const;
private:
  std::unique_ptr<fspp::Node> Load(const bf::path &path) override;

  const bf::path _root_path;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

inline const bf::path &CryDevice::RootDir() const {
  return _root_path;
}

} /* namespace cryfs */

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
